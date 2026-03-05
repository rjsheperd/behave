;; lang/python.clj — Python Google-style docstring annotation support
;; Implements multimethod dispatch for :python language.

(require '[clojure.string :as str])

;; ---------------------------------------------------------------------------
;; Python parameter parsing helpers
;; ---------------------------------------------------------------------------

(defn- ^:private py-skip-param?
  "Return true if a Python parameter should be skipped in docs."
  [name]
  (contains? #{"self" "cls" "*args" "**kwargs" "/"} name))

(defn- ^:private py-parse-params
  "Parse Python function parameters."
  [param-str]
  (when-not (or (str/blank? param-str)
                (= (str/trim param-str) ""))
    (->> (str/split param-str #",")
         (mapv str/trim)
         (remove str/blank?)
         (remove #(py-skip-param? (str/trim (first (str/split % #"[=:]" 2)))))
         (mapv (fn [p]
                 (let [p (str/replace p #"\s*=.*$" "")
                       parts (str/split p #":\s*" 2)]
                   (if (= 2 (count parts))
                     {:name (str/trim (first parts))
                      :type (str/trim (second parts))}
                     {:name (str/trim p) :type ""}))))
         (filterv #(not (str/blank? (:name %)))))))

(defn- ^:private py-parse-return-type
  "Extract return type annotation from a Python function signature."
  [sig]
  (when-let [m (re-find #"->\s*(.+?)\s*:" sig)]
    (let [rt (str/trim (second m))]
      (when-not (or (str/blank? rt) (= rt "None"))
        rt))))

(defn- ^:private py-indent-level
  "Get the indentation of a line (number of leading spaces)."
  [line]
  (count (re-find #"^\s*" line)))

;; ---------------------------------------------------------------------------
;; Multimethod implementations
;; ---------------------------------------------------------------------------

(defmethod skip-construct? :python
  [_lang line]
  (let [trimmed (str/trim line)]
    (or (re-find #"def\s+__\w+__" trimmed)
        (re-find #"def\s+_[^_]" trimmed))))

(defmethod file-doc-pattern :python
  [_lang]
  #"^(?:\s*#.*\n)*\s*(?:\"\"\"|''')")

(defmethod member-insertion-mode :python
  [_lang]
  :block)

(defmethod scan-constructs :python
  [_lang lines _filename _ext]
  (let [results (atom [])
        n (count lines)
        ;; Track class scope via indentation
        class-indent (atom nil)]
    (loop [i 0]
      (when (< i n)
        (let [line (nth lines i)
              trimmed (str/trim line)
              indent (py-indent-level line)]

          ;; Reset class scope when we encounter something at class indent or less
          (when (and @class-indent
                     (<= indent @class-indent)
                     (not (str/blank? trimmed))
                     (not (re-find #"^\s*#" trimmed))
                     (not (re-find #"^\s*@" trimmed)))
            (reset! class-indent nil))

          (cond
            ;; Decorator — skip, the construct follows
            (re-find #"^\s*@" trimmed)
            nil

            ;; class
            (re-find #"^\s*class\s+(\w+)" trimmed)
            (let [name (second (re-find #"class\s+(\w+)" trimmed))]
              (swap! results conj {:kind :class :name name :line (inc i) :indent indent})
              (reset! class-indent indent))

            ;; async def / def
            (and (re-find #"^\s*(?:async\s+)?def\s+(\w+)" trimmed)
                 (not (skip-construct? :python trimmed)))
            (let [name (second (re-find #"def\s+(\w+)" trimmed))
                  ;; Join continuation lines until we see the closing ):
                  [joined _end-i] (loop [j i acc []]
                                    (if (>= j n)
                                      [(str/join " " acc) (dec j)]
                                      (let [l (nth lines j)
                                            acc' (conj acc (str/trim l))
                                            joined (str/join " " acc')]
                                        (if (re-find #":" joined)
                                          [joined j]
                                          (recur (inc j) acc')))))
                  param-m (re-find #"\(([^)]*)\)" joined)
                  params (when param-m (py-parse-params (second param-m)))
                  ret (py-parse-return-type joined)
                  is-method? (some? @class-indent)]
              (swap! results conj {:kind (if is-method? :method :function)
                                   :name name
                                   :line (inc i)
                                   :params (vec (or params []))
                                   :return-type ret
                                   :indent indent
                                   :raw-sig joined}))

            :else nil))
        (recur (inc i))))
    @results))

(defmethod find-documented-lines :python
  [_lang text]
  (let [lines (str/split-lines text)
        documented (atom #{})]
    ;; For each def/class line, look ahead for a docstring
    (doseq [[idx line] (map-indexed vector lines)]
      (let [trimmed (str/trim line)]
        (when (re-find #"^\s*(?:async\s+)?(?:def|class)\s+\w+" trimmed)
          ;; Look ahead for """ or '''
          (loop [j (inc idx)]
            (when (< j (count lines))
              (let [l (str/trim (nth lines j))]
                (cond
                  (str/blank? l) (recur (inc j))
                  (re-find #"^(?:\"\"\"|''')" l) (swap! documented conj (inc idx))
                  :else nil)))))))
    @documented))

(defmethod generate-comment :python
  [_lang {:keys [kind name params return-type indent] :as construct} filename]
  (let [base-indent (or indent 0)
        doc-indent (str/join (repeat (+ base-indent 4) " "))]
    (case kind
      :file
      ["\"\"\"TODO: describe this module.\"\"\""]

      :class
      [(str doc-indent "\"\"\"TODO: describe this class.\"\"\"")]

      (:function :method)
      (let [has-params? (seq params)
            has-return? (and return-type (not (str/blank? return-type)))
            ;; Simple case: no params, no return
            simple? (and (not has-params?) (not has-return?))]
        (if simple?
          [(str doc-indent "\"\"\"TODO: describe what this function does.\"\"\"")]
          (let [header [(str doc-indent "\"\"\"TODO: describe what this function does.")]
                param-section (when has-params?
                                (into [(str doc-indent "")
                                       (str doc-indent "Args:")]
                                      (mapv (fn [{:keys [name type]}]
                                              (str doc-indent "    " name
                                                   (if (and type (not (str/blank? type)))
                                                     (str " (" type ")")
                                                     "")
                                                   ": TODO: describe."))
                                            params)))
                return-section (when has-return?
                                 [(str doc-indent "")
                                  (str doc-indent "Returns:")
                                  (str doc-indent "    TODO: describe return value.")])
                footer [(str doc-indent "\"\"\"")]]
            (vec (concat header param-section return-section footer)))))

      ;; Default
      [(str doc-indent "\"\"\"TODO: describe.\"\"\"")])))
