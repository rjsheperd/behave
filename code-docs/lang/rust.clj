;; lang/rust.clj — Rust rustdoc annotation support
;; Implements multimethod dispatch for :rust language.

(require '[clojure.string :as str])

;; ---------------------------------------------------------------------------
;; Rust parameter parsing helpers
;; ---------------------------------------------------------------------------

(defn- ^:private rust-parse-params
  "Parse Rust function parameters, skipping self variants."
  [param-str]
  (when-not (or (str/blank? param-str)
                (= (str/trim param-str) ""))
    (->> (str/split param-str #",")
         (mapv str/trim)
         (remove str/blank?)
         (remove #(re-matches #"(?:&\s*)?(?:mut\s+)?self" %))
         (mapv (fn [p]
                 (let [p (str/replace p #"\s*=.*$" "")
                       parts (str/split p #":\s*" 2)]
                   (if (= 2 (count parts))
                     {:name (str/trim (first parts))
                      :type (str/trim (second parts))}
                     {:name (str/trim p) :type ""}))))
         (filterv #(not (str/blank? (:name %)))))))

(defn- ^:private rust-parse-return-type
  "Extract return type from a Rust function signature."
  [sig]
  (when-let [m (re-find #"->\s*(.+?)(?:\s*\{|\s*where|\s*;|\s*$)" sig)]
    (let [rt (str/trim (second m))]
      (when-not (str/blank? rt)
        rt))))

;; ---------------------------------------------------------------------------
;; Multimethod implementations
;; ---------------------------------------------------------------------------

(defmethod skip-construct? :rust
  [_lang line]
  false)

(defmethod file-doc-pattern :rust
  [_lang]
  #"^//!\s")

(defmethod member-insertion-mode :rust
  [_lang]
  :block)

(defmethod scan-constructs :rust
  [_lang lines _filename _ext]
  (let [results (atom [])
        n (count lines)
        brace-depth (atom 0)
        struct-depth (atom nil)]
    (loop [i 0]
      (when (< i n)
        (let [line (nth lines i)
              trimmed (str/trim line)
              prev-depth @brace-depth]

          ;; Track brace depth
          (when (str/includes? line "{")
            (swap! brace-depth + (count (re-seq #"\{" line))))
          (when (str/includes? line "}")
            (swap! brace-depth - (count (re-seq #"\}" line)))
            (when (and @struct-depth (<= @brace-depth @struct-depth))
              (reset! struct-depth nil)))

          (cond
            ;; fn (standalone, pub, async, unsafe, etc.)
            (re-find #"^\s*(?:pub(?:\s*\([^)]*\))?\s+)?(?:async\s+)?(?:unsafe\s+)?(?:extern\s+\"[^\"]*\"\s+)?fn\s+(\w+)" trimmed)
            (let [name (second (re-find #"fn\s+(\w+)" trimmed))
                  ;; Join continuation lines until we see { or ;
                  [joined _end-i] (loop [j i acc []]
                                    (if (>= j n)
                                      [(str/join " " acc) (dec j)]
                                      (let [l (str/trim (nth lines j))
                                            acc' (conj acc l)
                                            joined (str/join " " acc')]
                                        (if (or (str/includes? joined "{")
                                                (str/includes? joined ";"))
                                          [joined j]
                                          (recur (inc j) acc')))))
                  param-m (re-find #"\(([^)]*)\)" joined)
                  params (when param-m (rust-parse-params (second param-m)))
                  ret (rust-parse-return-type joined)
                  in-struct? (some? @struct-depth)]
              (swap! results conj {:kind (if in-struct? :method :function)
                                   :name name
                                   :line (inc i)
                                   :params (vec (or params []))
                                   :return-type ret
                                   :raw-sig joined}))

            ;; struct
            (re-find #"^\s*(?:pub(?:\s*\([^)]*\))?\s+)?struct\s+(\w+)" trimmed)
            (let [name (second (re-find #"struct\s+(\w+)" trimmed))]
              (swap! results conj {:kind :struct :name name :line (inc i)})
              (when (str/includes? trimmed "{")
                (reset! struct-depth prev-depth)))

            ;; enum
            (re-find #"^\s*(?:pub(?:\s*\([^)]*\))?\s+)?enum\s+(\w+)" trimmed)
            (let [name (second (re-find #"enum\s+(\w+)" trimmed))]
              (swap! results conj {:kind :enum :name name :line (inc i)}))

            ;; trait
            (re-find #"^\s*(?:pub(?:\s*\([^)]*\))?\s+)?(?:unsafe\s+)?trait\s+(\w+)" trimmed)
            (let [name (second (re-find #"trait\s+(\w+)" trimmed))]
              (swap! results conj {:kind :trait :name name :line (inc i)}))

            ;; impl
            (re-find #"^\s*impl(?:<[^>]*>)?\s+(?:(\w+)(?:<[^>]*>)?\s+for\s+)?(\w+)" trimmed)
            (let [m (re-find #"impl(?:<[^>]*>)?\s+(?:(\w+)(?:<[^>]*>)?\s+for\s+)?(\w+)" trimmed)
                  trait-name (nth m 1)
                  type-name (nth m 2)
                  display (if trait-name
                            (str trait-name " for " type-name)
                            type-name)]
              (swap! results conj {:kind :impl :name display :line (inc i)})
              (reset! struct-depth prev-depth))

            ;; type alias
            (re-find #"^\s*(?:pub(?:\s*\([^)]*\))?\s+)?type\s+(\w+)" trimmed)
            (let [name (second (re-find #"type\s+(\w+)" trimmed))]
              (swap! results conj {:kind :type :name name :line (inc i)}))

            ;; const / static
            (re-find #"^\s*(?:pub(?:\s*\([^)]*\))?\s+)?(?:const|static)\s+(\w+)" trimmed)
            (let [name (second (re-find #"(?:const|static)\s+(\w+)" trimmed))]
              (when (not= name "_")
                (swap! results conj {:kind :const :name name :line (inc i)})))

            ;; mod
            (re-find #"^\s*(?:pub(?:\s*\([^)]*\))?\s+)?mod\s+(\w+)" trimmed)
            (let [name (second (re-find #"mod\s+(\w+)" trimmed))]
              (swap! results conj {:kind :module :name name :line (inc i)}))

            ;; struct field (inside struct body)
            (and (some? @struct-depth)
                 (re-find #"^\s*(?:pub(?:\s*\([^)]*\))?\s+)?(\w+)\s*:\s*(.+?)\s*,?\s*$" trimmed)
                 (not (re-find #"^\s*//" trimmed))
                 (not (re-find #"^\s*#" trimmed)))
            (let [m (re-find #"^\s*(?:pub(?:\s*\([^)]*\))?\s+)?(\w+)\s*:\s*(.+?)\s*,?\s*$" trimmed)]
              (when (and m (not (#{"fn" "let" "mut" "pub" "use" "mod" "type" "const" "static" "struct" "enum" "trait" "impl" "where" "async" "unsafe" "extern"} (nth m 1))))
                (swap! results conj {:kind :member
                                     :name (nth m 1)
                                     :type (str/replace (nth m 2) #",\s*$" "")
                                     :line (inc i)})))

            :else nil))
        (recur (inc i))))
    @results))

(defmethod find-documented-lines :rust
  [_lang text]
  (let [lines (str/split-lines text)
        documented (atom #{})]
    (doseq [[idx _line] (map-indexed vector lines)]
      ;; Look backward from each line: if preceded by /// lines, it's documented
      (let [trimmed (str/trim (nth lines idx))]
        (when (and (not (str/blank? trimmed))
                   (not (re-find #"^\s*//" trimmed))
                   (not (re-find #"^\s*#\[" trimmed)))
          (loop [j (dec idx)]
            (when (>= j 0)
              (let [prev (str/trim (nth lines j))]
                (cond
                  (or (str/blank? prev)
                      (re-find #"^\s*#\[" prev))
                  (recur (dec j))

                  (re-find #"^\s*///" prev)
                  (swap! documented conj (inc idx))

                  :else nil)))))))
    @documented))

(defmethod generate-comment :rust
  [_lang {:keys [kind name params return-type type] :as construct} filename]
  (case kind
    :file
    ["//! TODO: describe this module."]

    (:struct :enum)
    [(str "/// TODO: describe this " (clojure.core/name kind) ".")]

    :trait
    ["/// TODO: describe this trait."]

    :impl
    [(str "/// `" name "` implementation.")]

    :type
    ["/// TODO: describe this type alias."]

    :const
    ["/// TODO: describe this constant."]

    :module
    ["/// TODO: describe this module."]

    (:function :method)
    (let [header ["/// TODO: describe what this function does."]
          param-section (when (seq params)
                          (into ["///"
                                 "/// # Arguments"
                                 "///"]
                                (mapv (fn [{:keys [name type]}]
                                        (str "/// * `" name "` - TODO: describe"
                                             (when (and type (not (str/blank? type)))
                                               (str " (" type ")"))
                                             "."))
                                      params)))
          return-section (when (and return-type
                                    (not (str/blank? return-type))
                                    (not= return-type "()")
                                    (not (re-find #"^Result<\s*\(\)" return-type)))
                           ["///"
                            "/// # Returns"
                            "///"
                            "/// TODO: describe return value."])
          all (vec (concat header param-section return-section))]
      all)

    :member
    [(str "/// TODO: describe (" (or type "?") ").")]

    []))
