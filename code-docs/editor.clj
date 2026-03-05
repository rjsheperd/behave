;; editor.clj — Language-agnostic annotation insertion engine
;; Loaded by server.clj; dispatches to lang/*.clj via multimethods.

(require '[clojure.string :as str]
         '[clojure.java.io :as io])

;; ---------------------------------------------------------------------------
;; Utilities
;; ---------------------------------------------------------------------------

(defn- ^:private sha256
  "Compute SHA-256 hex digest of a string."
  [s]
  (let [md (java.security.MessageDigest/getInstance "SHA-256")
        bytes (.digest md (.getBytes s "UTF-8"))]
    (apply str (map #(format "%02x" %) bytes))))

;; ---------------------------------------------------------------------------
;; Insertion computation (language-agnostic orchestration)
;; ---------------------------------------------------------------------------

(defn compute-insertions
  "Compute insertion records for all undocumented constructs in a file.
   Returns a vector of insertion records sorted by line ascending."
  [folder file ext]
  (let [lang (lang-for-ext ext)
        _ (when-not lang
            (throw (ex-info "Unsupported file extension" {:ext ext})))
        src-root (io/file (.getParentFile (io/file *file*)) ".." "src")
        path (io/file src-root folder (str file ext))
        _ (when-not (.exists path)
            (throw (ex-info "File not found" {:path (str path)})))
        text (slurp path)
        lines (str/split-lines text)
        filename (str file ext)
        constructs (scan-constructs lang lines filename ext)
        documented (find-documented-lines lang text)
        has-file-doc? (boolean (re-find (file-doc-pattern lang) text))
        mode (member-insertion-mode lang)
        insertions (atom [])]

    ;; File-level doc
    (when (and (not has-file-doc?)
               (or (contains? #{".h" ".hpp"} ext)   ;; C++ headers
                   (= ext ".rs")                      ;; Rust files
                   (= ext ".py")))                    ;; Python files
      (let [insert-line (or (some (fn [[idx line]]
                                    (when (and (not (re-find #"^\s*/?\*" line))
                                               (not (re-find #"^\s*//" line))
                                               (not (re-find #"^\s*#" line))
                                               (not (str/blank? line)))
                                      (inc idx)))
                                  (map-indexed vector lines))
                            1)]
        (swap! insertions conj
               {:before-line insert-line
                :kind :file
                :target-name filename
                :lines (generate-comment lang {:kind :file} filename)})))

    ;; Process each construct
    (doseq [{:keys [kind name line] :as construct} constructs]
      (when-not (documented line)
        (let [comment-lines (generate-comment lang construct filename)]
          (if (and (= kind :member) (= mode :trailing))
            ;; Trailing comment for members (C++ style)
            (let [actual-line (nth lines (dec line))]
              (when-not (re-find #"//!<" actual-line)
                (swap! insertions conj
                       {:before-line line
                        :kind :member
                        :target-name name
                        :lines comment-lines})))
            ;; Block insertion — check if already documented by looking backward
            (let [has-doc? (loop [i (- line 2)]  ;; line is 1-indexed, look above
                             (when (>= i 0)
                               (let [l (str/trim (nth lines i))]
                                 (cond
                                   (or (str/blank? l)
                                       (re-matches #"^//[-=]+\s*$" l))
                                   (recur (dec i))

                                   ;; C++ doxygen block end
                                   (and (= lang :cpp) (re-find #"\*/" l))
                                   (boolean (re-find #"/\*!"
                                                     (str/join "\n" (subvec (vec lines)
                                                                            (max 0 (- i 20))
                                                                            (inc i)))))

                                   ;; Rust doc comments
                                   (and (= lang :rust) (re-find #"^\s*///" l))
                                   true

                                   ;; Python docstrings (look forward from def line instead)
                                   (and (= lang :python))
                                   false

                                   :else false))))]
              ;; For Python, also check forward for existing docstring
              (let [has-doc? (if (and (= lang :python) (not has-doc?))
                               ;; Look at next non-blank line after the construct
                               (let [next-idx line]  ;; line is 1-indexed, so (line) is the line after
                                 (loop [j next-idx]
                                   (if (>= j (count lines))
                                     false
                                     (let [l (str/trim (nth lines j))]
                                       (cond
                                         (str/blank? l) (recur (inc j))
                                         (re-find #"^(?:\"\"\"|''')" l) true
                                         :else false)))))
                               has-doc?)]
                (when-not has-doc?
                  (let [insert-line (if (= lang :python)
                                      (inc line)  ;; Python: docstring goes inside the body
                                      line)]      ;; Others: comment goes before the construct
                    (swap! insertions conj
                           {:before-line insert-line
                            :kind kind
                            :target-name name
                            :lines comment-lines})))))))))

    (vec (sort-by :before-line @insertions))))

(defn preview-insertions
  "Preview insertions for a file. Returns map with :insertions, :stats, :hash."
  [folder file ext]
  (let [lang (lang-for-ext ext)
        _ (when-not lang
            (throw (ex-info "Unsupported file extension" {:ext ext})))
        src-root (io/file (.getParentFile (io/file *file*)) ".." "src")
        path (io/file src-root folder (str file ext))
        text (slurp path)
        lines (str/split-lines text)
        insertions (compute-insertions folder file ext)
        constructs (scan-constructs lang lines (str file ext) ext)
        documented (find-documented-lines lang text)]
    {:file (str folder "/" file ext)
     :hash (sha256 text)
     :stats {:total (count constructs)
             :documented (count (filter #(documented (:line %)) constructs))
             :to-annotate (count insertions)}
     :insertions insertions}))

(defn apply-insertions!
  "Apply selected insertions to a file. Bottom-up to avoid offset shifting.
   Verifies content hash, creates .bak backup."
  [folder file ext insertions expected-hash]
  (let [src-root (io/file (.getParentFile (io/file *file*)) ".." "src")
        path (io/file src-root folder (str file ext))
        text (slurp path)
        current-hash (sha256 text)
        lang (lang-for-ext ext)
        mode (when lang (member-insertion-mode lang))]
    (when (not= current-hash expected-hash)
      (throw (ex-info "File has changed since preview"
                      {:expected expected-hash :actual current-hash})))
    (spit (str path ".bak") text)
    (let [lines (vec (str/split-lines text))
          sorted (reverse (sort-by :before-line insertions))
          result (reduce
                  (fn [ls {:keys [before-line kind lines]}]
                    (if (and (= kind :member) (= mode :trailing))
                      ;; Trailing comment — modify the existing line
                      (let [idx (dec before-line)
                            existing (nth ls idx)
                            trailing (first lines)
                            new-line (if (re-find #"//!<" existing)
                                       existing
                                       (let [base (str/replace existing #"\s*//[^!].*$" "")
                                             padded (if (str/ends-with? (str/trimr base) ";")
                                                      base
                                                      base)]
                                         (str (str/trimr padded) " " trailing)))]
                        (assoc ls idx new-line))
                      ;; Block insertion — insert before the target line
                      (let [idx (dec before-line)]
                        (vec (concat (subvec ls 0 idx)
                                     lines
                                     [""]
                                     (subvec ls idx))))))
                  lines
                  sorted)]
      (spit path (str/join "\n" result))
      {:status "ok"
       :lines-added (- (count result) (count lines))
       :backup (str path ".bak")})))
