;; lang/cpp.clj — C++ Doxygen annotation support
;; Implements multimethod dispatch for :cpp language.

(require '[clojure.string :as str])

;; ---------------------------------------------------------------------------
;; Separator line constant
;; ---------------------------------------------------------------------------

(def ^:private cpp-separator
  "//------------------------------------------------------------------------------")

;; ---------------------------------------------------------------------------
;; Template definitions
;; ---------------------------------------------------------------------------

(defn- ^:private cpp-file-template
  [{:keys [filename author brief]}]
  [cpp-separator
   (str "/*! \\file " filename)
   (str "    \\author " (or author "TODO: author name"))
   (str "    \\brief " (or brief "TODO: describe this file."))
   " */"])

(defn- ^:private cpp-class-template
  [{:keys [class-name filename brief]}]
  [cpp-separator
   (str "/*! \\class " class-name " " filename)
   (str "    \\brief " (or brief "TODO: describe this class."))
   " */"])

(defn- ^:private cpp-struct-template
  [{:keys [struct-name filename brief]}]
  [cpp-separator
   (str "/*! \\class " struct-name " " filename)
   (str "    \\brief " (or brief "TODO: describe this struct."))
   " */"])

(defn- ^:private cpp-enum-template
  [{:keys [enum-name brief]}]
  [cpp-separator
   (str "/*! \\enum " enum-name)
   (str "    \\brief " (or brief "TODO: describe this enumeration."))
   " */"])

(defn- ^:private cpp-format-param
  [{:keys [name type]}]
  (str "    \\param[in] " name " TODO: describe"
       (when (and type (not (str/blank? type)))
         (str " (" type ")"))
       "."))

(defn- ^:private cpp-function-template
  [{:keys [brief params return-type]}]
  (let [header [cpp-separator
                (str "/*! \\brief " (or brief "TODO: describe what this function does."))]
        param-lines (when (seq params)
                      (into [""] (mapv cpp-format-param params)))
        return-line (when (and return-type
                               (not= return-type "void")
                               (not (str/blank? return-type)))
                      ["" "    \\return TODO: describe return value."])
        footer [" */"]]
    (vec (concat header param-lines return-line footer))))

(defn- ^:private cpp-member-trailing-comment
  [{:keys [type]}]
  (str "//!< TODO: describe (" (or type "?") ")"))

;; ---------------------------------------------------------------------------
;; C++ signature parsing helpers
;; ---------------------------------------------------------------------------

(defn- ^:private join-continuation-lines
  [lines idx]
  (loop [i idx
         acc []]
    (if (>= i (count lines))
      [(str/join " " acc) (dec i)]
      (let [line (str/trim (nth lines i))
            acc' (conj acc line)
            joined (str/join " " acc')]
        (if (or (str/includes? joined ")")
                (str/includes? joined ";"))
          [joined i]
          (recur (inc i) acc'))))))

(defn- ^:private parse-function-params
  [sig]
  (when-let [m (re-find #"\(([^)]*)\)" sig)]
    (let [param-str (second m)]
      (when-not (or (str/blank? param-str)
                    (= (str/trim param-str) "void")
                    (= (str/trim param-str) ""))
        (->> (str/split param-str #",")
             (mapv str/trim)
             (remove str/blank?)
             (mapv (fn [p]
                     (let [p (str/replace p #"\s*=.*$" "")
                           tokens (str/split (str/trim p) #"\s+")
                           name (last tokens)
                           clean-name (str/replace (or name "") #"^[*&]+" "")
                           type-tokens (butlast tokens)
                           type-str (str/join " " type-tokens)]
                       {:name clean-name :type type-str})))
             (filterv #(not (str/blank? (:name %)))))))))

(defn- ^:private parse-return-type
  [sig]
  (let [clean (str/replace sig #"<[^>]+>" "")
        m (re-find #"^\s*([\w:*&<>\s]+?)\s+\~?\w+(?:::\~?\w+)*\s*\(" clean)]
    (when m
      (let [rt (str/trim (second m))]
        (when-not (or (str/blank? rt)
                      (#{"public" "private" "protected" "static" "virtual" "inline" "explicit"} rt))
          rt)))))

;; ---------------------------------------------------------------------------
;; Doxygen detection helpers
;; ---------------------------------------------------------------------------

(defn- ^:private line-has-doxygen-before?
  [lines idx]
  (loop [i (dec idx)]
    (when (>= i 0)
      (let [line (str/trim (nth lines i))]
        (cond
          (or (str/blank? line)
              (re-matches #"^//[-=]+\s*$" line))
          (recur (dec i))

          (re-find #"\*/" line)
          (boolean (re-find #"/\*!" (str/join "\n" (subvec (vec lines) (max 0 (- i 20)) (inc i)))))

          (re-find #"^//" line)
          false

          :else
          false)))))

(defn- ^:private line-has-trailing-doxygen?
  [line]
  (boolean (re-find #"//!<" line)))

;; ---------------------------------------------------------------------------
;; Multimethod implementations
;; ---------------------------------------------------------------------------

(defmethod skip-construct? :cpp
  [_lang line]
  (or (re-find #"operator\s*=" line)
      (re-find #"~\w+" line)
      (re-find #"=\s*delete" line)))

(defmethod file-doc-pattern :cpp
  [_lang]
  #"/\*!\s*\\file")

(defmethod member-insertion-mode :cpp
  [_lang]
  :trailing)

(defmethod scan-constructs :cpp
  [_lang lines _filename ext]
  (let [results (atom [])
        n (count lines)
        header? (contains? #{".h" ".hpp"} ext)
        brace-depth (atom 0)
        in-class? (atom false)
        class-brace-start (atom 0)
        next-i (atom nil)]
    (loop [i 0]
      (when (< i n)
        (reset! next-i nil)
        (let [line (nth lines i)
              trimmed (str/trim line)]
          (when (str/includes? line "{")
            (let [opens (count (re-seq #"\{" line))]
              (swap! brace-depth + opens)))
          (when (str/includes? line "}")
            (let [closes (count (re-seq #"\}" line))]
              (swap! brace-depth - closes)
              (when (and @in-class? (<= @brace-depth @class-brace-start))
                (reset! in-class? false))))

          (cond
            ;; Struct
            (and header?
                 (re-find #"^\s*struct\s+(\w+)" trimmed)
                 (not (re-find #";\s*$" trimmed)))
            (let [name (second (re-find #"^\s*struct\s+(\w+)" trimmed))]
              (swap! results conj {:kind :struct :name name :line (inc i)})
              (reset! in-class? true)
              (reset! class-brace-start @brace-depth))

            ;; Class
            (and header?
                 (re-find #"^\s*class\s+(\w+)" trimmed)
                 (not (re-find #";\s*$" trimmed))
                 (not (re-find #"^\s*class\s+\w+\s*;" trimmed)))
            (let [name (second (re-find #"^\s*class\s+(\w+)" trimmed))]
              (swap! results conj {:kind :class :name name :line (inc i)})
              (reset! in-class? true)
              (reset! class-brace-start @brace-depth))

            ;; Enum
            (and header?
                 (re-find #"^\s*enum\s+(?:class\s+)?(\w+)" trimmed))
            (let [name (second (re-find #"^\s*enum\s+(?:class\s+)?(\w+)" trimmed))]
              (swap! results conj {:kind :enum :name name :line (inc i)}))

            ;; Function in .cpp
            (and (not header?)
                 (re-find #"(\w+)::(\~?\w+)\s*\(" trimmed)
                 (not (skip-construct? :cpp trimmed)))
            (let [[joined end-i] (join-continuation-lines lines i)
                  m (re-find #"((?:\w+::)*\~?\w+)\s*\(" joined)
                  func-name (when m (second m))
                  params (parse-function-params joined)
                  ret (parse-return-type joined)]
              (when (and func-name (not (skip-construct? :cpp joined)))
                (swap! results conj {:kind :function
                                     :name func-name
                                     :line (inc i)
                                     :params (vec (or params []))
                                     :return-type ret
                                     :raw-sig joined}))
              (reset! next-i (inc end-i)))

            ;; Method declaration in .h
            (and header?
                 @in-class?
                 (re-find #"\(" trimmed)
                 (not (skip-construct? :cpp trimmed))
                 (re-find #"^\s*(?:virtual\s+|static\s+|inline\s+|explicit\s+)*[\w:*&<>]+\s+\w+\s*\(" trimmed))
            (let [[joined end-i] (join-continuation-lines lines i)
                  m (re-find #"(\w+)\s*\(" joined)
                  func-name (when m (second m))
                  params (parse-function-params joined)
                  ret (parse-return-type joined)]
              (when (and func-name
                         (not (skip-construct? :cpp joined))
                         (not (#{"if" "while" "for" "switch" "return" "catch"} func-name)))
                (swap! results conj {:kind :function
                                     :name func-name
                                     :line (inc i)
                                     :params (vec (or params []))
                                     :return-type ret
                                     :raw-sig joined}))
              (reset! next-i (inc end-i)))

            ;; Member variable in .h
            (and header?
                 @in-class?
                 (re-find #"^\s+[\w:<>,\s\*&]+\s+\w+_?\s*[;=]" trimmed)
                 (not (re-find #"\(" trimmed))
                 (not (re-find #"^\s*class\s" trimmed))
                 (not (re-find #"^\s*struct\s" trimmed))
                 (not (re-find #"^\s*enum\s" trimmed))
                 (not (re-find #"^\s*//" trimmed)))
            (let [m (re-find #"^\s+([\w:<>,\s\*&]+?)\s+(\w+)\s*[;=]" trimmed)]
              (when m
                (swap! results conj {:kind :member
                                     :name (str/trim (nth m 2))
                                     :type (str/trim (nth m 1))
                                     :line (inc i)})))

            :else nil))
        (recur (if @next-i @next-i (inc i)))))
    @results))

(defmethod find-documented-lines :cpp
  [_lang text]
  (let [lines (str/split-lines text)
        documented (atom #{})]
    (let [in-block? (atom false)
          block-end (atom 0)]
      (doseq [[idx line] (map-indexed vector lines)]
        (cond
          (and (not @in-block?) (re-find #"/\*!" line))
          (reset! in-block? true)

          (and @in-block? (re-find #"\*/" line))
          (do (reset! in-block? false)
              (reset! block-end idx)
              (loop [j (inc @block-end)]
                (when (< j (count lines))
                  (let [l (str/trim (nth lines j))]
                    (cond
                      (or (str/blank? l)
                          (re-matches #"^//[-=].*" l))
                      (recur (inc j))

                      :else
                      (swap! documented conj (inc j))))))))))
    (doseq [[idx line] (map-indexed vector lines)]
      (when (re-find #"//!<" line)
        (swap! documented conj (inc idx))))
    @documented))

(defmethod generate-comment :cpp
  [_lang {:keys [kind name params return-type type] :as construct} filename]
  (case kind
    :file     (cpp-file-template {:filename filename})
    :class    (cpp-class-template {:class-name name :filename filename})
    :struct   (cpp-struct-template {:struct-name name :filename filename})
    :enum     (cpp-enum-template {:enum-name name})
    :function (cpp-function-template {:params params :return-type return-type})
    :member   [(cpp-member-trailing-comment {:name name :type type})]
    []))
