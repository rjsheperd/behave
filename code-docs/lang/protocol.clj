;; lang/protocol.clj — Multi-language dispatch protocol for code annotation
;; Loaded by server.clj; all defs in user namespace.

(def ext->lang
  "Map file extensions to language keywords."
  {".h" :cpp ".hpp" :cpp ".cpp" :cpp ".cc" :cpp
   ".rs" :rust ".py" :python})

(defn lang-for-ext
  "Return the language keyword for a file extension, or nil."
  [ext]
  (get ext->lang ext))

(defmulti scan-constructs
  "Scan source lines and identify all documentable constructs.
   Returns a vector of maps: {:kind :name :line :params :return-type ...}
   where :line is 1-indexed."
  (fn [lang _lines _filename _ext] lang))

(defmulti find-documented-lines
  "Find the set of 1-indexed line numbers that already have documentation.
   Returns a set of integers."
  (fn [lang _text] lang))

(defmulti generate-comment
  "Generate documentation comment lines for a construct.
   Returns a vector of strings."
  (fn [lang _construct _filename] lang))

(defmulti skip-construct?
  "Return true if this line represents a construct that should be skipped."
  (fn [lang _line] lang))

(defmulti file-doc-pattern
  "Return a regex pattern that matches file-level documentation."
  (fn [lang] lang))

(defmulti member-insertion-mode
  "Return :trailing (C++) or :block (Rust, Python) for member doc placement."
  (fn [lang] lang))
