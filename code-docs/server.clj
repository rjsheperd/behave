#!/usr/bin/env bb

(require '[org.httpkit.server :as http]
         '[clojure.java.io :as io]
         '[cheshire.core :as json]
         '[clojure.string :as str])

(def port (or (some-> (first *command-line-args*) parse-long) 8080))
(def project-root (.getParentFile (io/file *file*)))
(def src-root (io/file (.getParentFile project-root) "src"))
(def org-file (io/file project-root "behave.org"))

;; Load language protocol and implementations
(load-file (str (.getAbsolutePath project-root) "/lang/protocol.clj"))
(load-file (str (.getAbsolutePath project-root) "/editor.clj"))
(load-file (str (.getAbsolutePath project-root) "/lang/cpp.clj"))
(load-file (str (.getAbsolutePath project-root) "/lang/rust.clj"))
(load-file (str (.getAbsolutePath project-root) "/lang/python.clj"))

;; --- MIME types ---

(def ^:private mime-types
  {"html" "text/html"
   "css"  "text/css"
   "js"   "application/javascript"
   "json" "application/json"
   "h"    "text/plain"
   "hpp"  "text/plain"
   "cpp"  "text/plain"
   "cc"   "text/plain"
   "rs"   "text/x-rust"
   "py"   "text/x-python"
   "png"  "image/png"
   "svg"  "image/svg+xml"
   "ico"  "image/x-icon"})

(defn- mime-type [path]
  (let [ext (last (str/split path #"\."))]
    (get mime-types ext "application/octet-stream")))

;; --- HTML -> Org conversion ---

(defn- strip-tags [html]
  (str/replace html #"<[^>]*>" ""))

(defn- html->org
  "Convert Quill HTML to Org-mode markup."
  [html]
  (when (and html (not (str/blank? html)))
    (-> html
        ;; Code blocks
        (str/replace #"<pre class=\"ql-syntax[^\"]*\"[^>]*>([\s\S]*?)</pre>"
                     (fn [[_ code]]
                       (str "\n#+BEGIN_SRC\n" (strip-tags code) "\n#+END_SRC\n")))
        ;; Headings
        (str/replace #"<h1[^>]*>(.*?)</h1>" (fn [[_ t]] (str "\n** " (strip-tags t) "\n")))
        (str/replace #"<h2[^>]*>(.*?)</h2>" (fn [[_ t]] (str "\n*** " (strip-tags t) "\n")))
        (str/replace #"<h3[^>]*>(.*?)</h3>" (fn [[_ t]] (str "\n**** " (strip-tags t) "\n")))
        ;; Blockquote
        (str/replace #"<blockquote[^>]*>([\s\S]*?)</blockquote>"
                     (fn [[_ t]] (str "\n#+BEGIN_QUOTE\n" (strip-tags t) "\n#+END_QUOTE\n")))
        ;; Lists - unordered
        (str/replace #"<ul>([\s\S]*?)</ul>"
                     (fn [[_ items]]
                       (str "\n"
                            (str/replace items #"<li[^>]*>(.*?)</li>"
                                         (fn [[_ t]] (str "- " (strip-tags t) "\n")))
                            "\n")))
        ;; Lists - ordered
        (str/replace #"<ol>([\s\S]*?)</ol>"
                     (fn [[_ items]]
                       (let [counter (atom 0)]
                         (str "\n"
                              (str/replace items #"<li[^>]*>(.*?)</li>"
                                           (fn [[_ t]]
                                             (swap! counter inc)
                                             (str @counter ". " (strip-tags t) "\n")))
                              "\n"))))
        ;; Links
        (str/replace #"<a[^>]*href=\"([^\"]+)\"[^>]*>(.*?)</a>"
                     (fn [[_ url text]] (str "[[" url "][" (strip-tags text) "]]")))
        ;; Inline formatting
        (str/replace #"<strong>(.*?)</strong>" (fn [[_ t]] (str "*" t "*")))
        (str/replace #"<b>(.*?)</b>" (fn [[_ t]] (str "*" t "*")))
        (str/replace #"<em>(.*?)</em>" (fn [[_ t]] (str "/" t "/")))
        (str/replace #"<i>(.*?)</i>" (fn [[_ t]] (str "/" t "/")))
        (str/replace #"<u>(.*?)</u>" (fn [[_ t]] (str "_" t "_")))
        (str/replace #"<s>(.*?)</s>" (fn [[_ t]] (str "+" t "+")))
        (str/replace #"<code>(.*?)</code>" (fn [[_ t]] (str "~" t "~")))
        ;; Strip remaining tags, normalize whitespace
        (str/replace #"<br\s*/?>" "\n")
        (str/replace #"<p[^>]*>" "\n")
        (str/replace #"</p>" "")
        (str/replace #"<[^>]*>" "")
        (str/replace #"&amp;" "&")
        (str/replace #"&lt;" "<")
        (str/replace #"&gt;" ">")
        (str/replace #"&nbsp;" " ")
        (str/replace #"\n{3,}" "\n\n")
        str/trim)))

;; --- Org -> HTML conversion ---

(defn- inline-org->html
  "Convert Org inline markup to HTML within a text string (no HTML tags present)."
  [text]
  (-> text
      (str/replace #"\*([^\*]+)\*" "<strong>$1</strong>")
      (str/replace #"/([^/]+)/" "<em>$1</em>")
      (str/replace #"_([^_]+)_" "<u>$1</u>")
      (str/replace #"\+([^\+]+)\+" "<s>$1</s>")
      (str/replace #"~([^~]+)~" "<code>$1</code>")
      (str/replace #"\[\[([^\]]+)\]\[([^\]]+)\]\]" "<a href=\"$1\">$2</a>")))

(defn- org->html
  "Convert Org-mode section body back to HTML for Quill."
  [org-text]
  (when (and org-text (not (str/blank? org-text)))
    (let [lines     (str/split-lines org-text)
          sb        (StringBuilder.)
          in-src?   (atom false)
          in-quote? (atom false)]
      (doseq [line lines]
        (cond
          (str/starts-with? line "#+BEGIN_SRC")
          (do (reset! in-src? true)
              (.append sb "<pre class=\"ql-syntax\" spellcheck=\"false\">"))

          (str/starts-with? line "#+END_SRC")
          (do (reset! in-src? false)
              (.append sb "</pre>"))

          @in-src?
          (.append sb (str line "\n"))

          (str/starts-with? line "#+BEGIN_QUOTE")
          (do (reset! in-quote? true)
              (.append sb "<blockquote>"))

          (str/starts-with? line "#+END_QUOTE")
          (do (reset! in-quote? false)
              (.append sb "</blockquote>"))

          (re-matches #"^\*\*\*\* (.+)" line)
          (.append sb (str "<h3>" (inline-org->html (second (re-matches #"^\*\*\*\* (.+)" line))) "</h3>"))

          (re-matches #"^\*\*\* (.+)" line)
          (.append sb (str "<h2>" (inline-org->html (second (re-matches #"^\*\*\* (.+)" line))) "</h2>"))

          (re-matches #"^\*\* (.+)" line)
          (.append sb (str "<h1>" (inline-org->html (second (re-matches #"^\*\* (.+)" line))) "</h1>"))

          (re-matches #"^- (.+)" line)
          (.append sb (str "<ul><li>" (inline-org->html (second (re-matches #"^- (.+)" line))) "</li></ul>"))

          (re-matches #"^\d+\. (.+)" line)
          (.append sb (str "<ol><li>" (inline-org->html (second (re-matches #"^\d+\. (.+)" line))) "</li></ol>"))

          :else
          (when-not (str/blank? line)
            (.append sb (str "<p>" (inline-org->html line) "</p>")))))
      (-> (str sb)
          ;; Merge adjacent list elements
          (str/replace #"</ul><ul>" "")
          (str/replace #"</ol><ol>" "")))))

;; --- Org file I/O ---

(defn- parse-org-file
  "Parse behave.org into a map of {file-key html-content}."
  []
  (if (.exists org-file)
    (let [content  (slurp org-file)
          sections (re-seq #"(?m)^\* (.+)\n([\s\S]*?)(?=^\* |\z)" content)]
      (into {}
            (for [[_ heading body] sections]
              [(str/trim heading) (org->html (str/trim body))])))
    {}))

(defn- write-org-file!
  "Write docs map to behave.org."
  [docs-map]
  (let [sorted  (sort-by key docs-map)
        content (str "#+TITLE: BehavePlus Code Documentation\n"
                     "#+AUTHOR: BehavePlus Team\n"
                     "#+DATE: " (.format (java.time.LocalDate/now) java.time.format.DateTimeFormatter/ISO_LOCAL_DATE) "\n\n"
                     (str/join "\n"
                               (for [[file-key html] sorted
                                     :when           (and html (not (str/blank? (strip-tags html))))]
                                 (str "* " file-key "\n"
                                      (html->org html) "\n"))))]
    (spit org-file content)
    (println "Wrote" org-file)))

;; --- Doxygen parser ---

(defn- ^:private read-source-text
  "Read a source file's text, returning nil if not found."
  [folder file ext]
  (let [path (io/file src-root folder (str file ext))]
    (when (.exists path)
      (slurp path))))

(defn- ^:private parse-doxygen-block
  "Parse a single /*! ... */ block into a map of {:brief :params :returns :class :enum :file :body}."
  [block-text]
  (let [lines   (str/split-lines block-text)
        brief   (atom nil)
        params  (atom [])
        returns (atom nil)
        klass   (atom nil)
        enum-n  (atom nil)
        file-n  (atom nil)
        body    (atom [])]
    (doseq [line lines]
      (let [trimmed (str/trim line)]
        (cond
          (re-find #"\\brief\s+" trimmed)
          (reset! brief (str/trim (second (re-find #"\\brief\s+(.*)" trimmed))))

          (re-find #"\\param(?:\[(in|out|in,out)\])?\s+" trimmed)
          (let [[_ dir name doc] (re-find #"\\param(?:\[(in|out|in,out)\])?\s+(\w+)\s*(.*)" trimmed)]
            (swap! params conj {:name name :dir (or dir "in") :doc (str/trim (or doc ""))}))

          (re-find #"\\returns?\s+" trimmed)
          (reset! returns (str/trim (second (re-find #"\\returns?\s+(.*)" trimmed))))

          (re-find #"\\retval\s+" trimmed)
          (reset! returns (str/trim (second (re-find #"\\retval\s+(.*)" trimmed))))

          (re-find #"\\class\s+" trimmed)
          (reset! klass (str/trim (second (re-find #"\\class\s+(\w+)" trimmed))))

          (re-find #"\\enum\s+" trimmed)
          (reset! enum-n (str/trim (second (re-find #"\\enum\s+(\w+)" trimmed))))

          (re-find #"\\file" trimmed)
          (reset! file-n true)

          ;; Accumulate non-tag body lines (skip comment delimiters)
          (not (or (re-find #"^\s*/\*!" trimmed)
                   (re-find #"^\s*\*/" trimmed)
                   (re-find #"^\s*\*?\s*$" trimmed)
                   (re-find #"^\\(brief|param|return|retval|class|enum|file|author|license)" trimmed)))
          (swap! body conj (str/replace trimmed #"^\s*\*?\s?" "")))))
    {:brief   @brief
     :params  @params
     :returns @returns
     :class   @klass
     :enum    @enum-n
     :file    @file-n
     :body    (when (seq @body) (str/join "\n" @body))}))

(defn- ^:private extract-doxygen-blocks
  "Extract all /*! ... */ blocks from source text with their ending line numbers."
  [text]
  (let [lines     (str/split-lines text)
        blocks    (atom [])
        in-block? (atom false)
        buf       (atom [])
        start-ln  (atom 0)]
    (doseq [[idx line] (map-indexed vector lines)]
      (cond
        (and (not @in-block?) (re-find #"/\*!" line))
        (do (reset! in-block? true)
            (reset! buf [line])
            (reset! start-ln (inc idx)))

        (and @in-block? (re-find #"\*/" line))
        (do (swap! buf conj line)
            (reset! in-block? false)
            (swap! blocks conj {:text    (str/join "\n" @buf)
                                :start   @start-ln
                                :end-line (inc idx)}))

        @in-block?
        (swap! buf conj line)))
    @blocks))

(defn- ^:private identify-code-after
  "Look at lines after a doc block to identify what it documents.
   Returns {:kind :name :line} or nil."
  [text-lines from-line]
  (loop [i from-line]
    (when (< i (count text-lines))
      (let [line (str/trim (nth text-lines i))]
        (cond
          ;; Skip blank lines and separator comments
          (or (str/blank? line)
              (re-matches #"^//[-=].*" line)
              (re-matches #"^/\*.*" line))
          (recur (inc i))

          ;; class definition
          (re-find #"^class\s+(\w+)" line)
          {:kind :class :name (second (re-find #"^class\s+(\w+)" line)) :line (inc i)}

          ;; enum inside struct or standalone
          (re-find #"^enum\s+(?:class\s+)?(\w+)" line)
          {:kind :enum :name (second (re-find #"^enum\s+(?:class\s+)?(\w+)" line)) :line (inc i)}

          ;; function definition (Namespace::Class::method or standalone)
          (re-find #"(\w+(?:::\~?\w+)+)\s*\(" line)
          (let [m (re-find #"((?:\w+::)*\~?\w+)\s*\(" line)]
            {:kind :function :name (second m) :line (inc i)})

          ;; simple function: return_type name(
          (re-find #"^[\w*&<>\s]+\s+(\w+)\s*\(" line)
          {:kind :function :name (second (re-find #"(\w+)\s*\(" line)) :line (inc i)}

          :else nil)))))

(defn- ^:private extract-enum-values
  "Extract enum values with trailing comments from lines following an enum declaration."
  [text-lines from-line]
  (let [values       (atom [])
        found-brace? (atom false)]
    (loop [i from-line]
      (when (< i (count text-lines))
        (let [line (nth text-lines i)]
          (cond
            (and (not @found-brace?) (str/includes? line "{"))
            (do (reset! found-brace? true) (recur (inc i)))

            (and @found-brace? (str/includes? line "}"))
            nil ;; done

            @found-brace?
            (do (when-let [m (re-find #"(\w+)\s*(?:=\s*\w+)?\s*,?\s*//[!<]*\s*(.*)" line)]
                  (swap! values conj {:name (nth m 1) :doc (str/trim (nth m 2))}))
                (recur (inc i)))

            :else (recur (inc i))))))
    @values))

(defn- ^:private find-class-body-members
  "Find member variables within a class body in the header."
  [h-lines class-start-line]
  (let [brace-depth (atom 0)
        in-class?   (atom false)
        members     (atom [])]
    (loop [i (dec class-start-line)] ;; 0-indexed
      (when (< i (count h-lines))
        (let [line (nth h-lines i)]
          (when (str/includes? line "{")
            (swap! brace-depth + (count (re-seq #"\{" line)))
            (reset! in-class? true))
          (when (str/includes? line "}")
            (swap! brace-depth - (count (re-seq #"\}" line))))
          (when (and @in-class? (pos? @brace-depth))
            ;; Look for member vars with trailing comments
            (when-let [m (re-find #"^\s+([\w:<>,\s\*&]+?)\s+(m_\w+|\w+_)\s*[;=].*?//[!<]*\s*(.*)" line)]
              (swap! members conj {:name (str/trim (nth m 2))
                                   :type (str/trim (nth m 1))
                                   :doc  (str/trim (nth m 3))
                                   :line (inc i)})))
          ;; Continue if we haven't started yet, or still inside the class
          (when (or (not @in-class?) (pos? @brace-depth))
            (recur (inc i))))))
    @members))

(defn- ^:private process-source-blocks
  "Process doxygen blocks from a single source text. Returns extracted items."
  [text source-lines]
  (let [blocks    (extract-doxygen-blocks text)
        file-doc  (atom nil)
        classes   (atom [])
        functions (atom [])
        enums     (atom [])]
    (doseq [block blocks]
      (let [parsed   (parse-doxygen-block (:text block))
            code-aft (identify-code-after source-lines (:end-line block))]
        (cond
          (:file parsed)
          (reset! file-doc (or (:brief parsed) (:body parsed) ""))

          (or (:class parsed) (= :class (:kind code-aft)))
          (let [class-name (or (:class parsed) (:name code-aft))
                class-line (or (:line code-aft) (:end-line block))
                members    (find-class-body-members source-lines class-line)]
            (swap! classes conj
                   {:name    class-name
                    :brief   (or (:brief parsed) "")
                    :members (vec members)}))

          (or (:enum parsed) (= :enum (:kind code-aft)))
          (let [enum-name (or (:enum parsed) (:name code-aft))
                vals-line (or (:line code-aft) (:end-line block))
                values    (extract-enum-values source-lines (dec vals-line))]
            (swap! enums conj
                   {:name   enum-name
                    :brief  (or (:brief parsed) "")
                    :values (vec values)}))

          (= :function (:kind code-aft))
          (swap! functions conj
                 {:name    (:name code-aft)
                  :brief   (or (:brief parsed) "")
                  :params  (:params parsed)
                  :returns (:returns parsed)
                  :line    (:line code-aft)})

          (and (:brief parsed) (nil? @file-doc))
          (reset! file-doc (:brief parsed)))))
    {:file-doc  @file-doc
     :classes   @classes
     :functions @functions
     :enums     @enums}))

(defn- parse-source-file
  "Parse C++ source files for Doxygen documentation.
   Returns structured data with classes, functions, enums, and file doc."
  [folder file]
  (let [h-text     (read-source-text folder file ".h")
        cpp-text   (read-source-text folder file ".cpp")
        h-lines    (when h-text (str/split-lines h-text))
        cpp-lines  (when cpp-text (str/split-lines cpp-text))
        h-result   (when h-text (process-source-blocks h-text h-lines))
        cpp-result (when cpp-text (process-source-blocks cpp-text cpp-lines))
        file-doc   (or (:file-doc h-result) (:file-doc cpp-result) "")
        classes    (vec (concat (:classes h-result) (:classes cpp-result)))
        functions  (vec (concat (:functions h-result) (:functions cpp-result)))
        enums      (vec (concat (:enums h-result) (:enums cpp-result)))]
    ;; Also extract enums with plain // comments (non-Doxygen, like crown.h)
    (let [extra-enums
          (when h-lines
            (let [existing-names (set (map :name enums))]
              (keep (fn [[idx line]]
                      (when-let [m (re-find #"^\s*enum\s+(?:class\s+)?(\w+)" line)]
                        (let [enum-name (second m)]
                          (when-not (existing-names enum-name)
                            (let [values (extract-enum-values h-lines idx)]
                              (when (seq values)
                                {:name enum-name :brief "" :values (vec values)}))))))
                    (map-indexed vector h-lines))))]
      {:file_doc  file-doc
       :classes   classes
       :functions functions
       :enums     (vec (concat enums extra-enums))})))

(defn- doxygen->org-html
  "Convert parsed Doxygen data to Quill-compatible HTML."
  [{:keys [file_doc classes functions enums]} folder file]
  (let [sb (StringBuilder.)]
    ;; File header
    (when (and file_doc (not (str/blank? file_doc)))
      (.append sb (str "<p>" file_doc "</p>")))

    ;; Classes
    (doseq [{:keys [name brief members]} classes]
      (.append sb (str "<h1>" name "</h1>"))
      (when (and brief (not (str/blank? brief)))
        (.append sb (str "<p>" brief "</p>")))
      (when (seq members)
        (.append sb "<h2>Member Variables</h2><ul>")
        (doseq [{:keys [name type doc line]} members]
          (let [ext  ".h"
                link (str "file:src/" folder "/" file ext "::" line)]
            (.append sb (str "<li><code>" name "</code>"
                             (when (and type (not (str/blank? type)))
                               (str " (" type ")"))
                             (when (and doc (not (str/blank? doc)))
                               (str " &mdash; " doc))
                             " <a href=\"" link "\">L" line "</a>"
                             "</li>"))))
        (.append sb "</ul>")))

    ;; Enums
    (when (seq enums)
      (.append sb "<h2>Enumerations</h2>")
      (doseq [{:keys [name brief values]} enums]
        (.append sb (str "<h3>" name "</h3>"))
        (when (and brief (not (str/blank? brief)))
          (.append sb (str "<p>" brief "</p>")))
        (when (seq values)
          (.append sb "<ul>")
          (doseq [{:keys [name doc]} values]
            (.append sb (str "<li><code>" name "</code>"
                             (when (and doc (not (str/blank? doc)))
                               (str " &mdash; " doc))
                             "</li>")))
          (.append sb "</ul>"))))

    ;; Functions
    (when (seq functions)
      (.append sb "<h2>Functions</h2>")
      (doseq [{:keys [name brief params returns line]} functions]
        (.append sb (str "<h3>" name "</h3>"))
        (when (and brief (not (str/blank? brief)))
          (.append sb (str "<p>" brief "</p>")))
        (when (seq params)
          (.append sb "<p><strong>Parameters:</strong></p><ul>")
          (doseq [{:keys [name dir doc]} params]
            (.append sb (str "<li><code>" name "</code>"
                             (when dir (str " (" dir ")"))
                             (when (and doc (not (str/blank? doc)))
                               (str " &mdash; " doc))
                             "</li>")))
          (.append sb "</ul>"))
        (when (and returns (not (str/blank? returns)))
          (.append sb (str "<p><strong>Returns:</strong> " returns "</p>")))
        (when line
          (let [ext  ".cpp"
                link (str "file:src/" folder "/" file ext "::" line)]
            (.append sb (str "<p><a href=\"" link "\">Source L" line "</a></p>"))))))
    (str sb)))

;; --- File search ---

(defn- find-source-file
  "Find a source file by name, searching recursively under src-root.
   Returns the relative path from src-root (e.g. \"behave/crown.h\") or nil."
  [filename]
  (let [results (atom [])]
    (doseq [f     (file-seq src-root)
            :when (and (.isFile f) (= (.getName f) filename))]
      (let [rel (.relativize (.toPath src-root) (.toPath f))]
        (swap! results conj (str rel))))
    (first @results)))

(defn- parse-src-uri
  "Parse a /src/ URI into {:file-path, :line-ref}.
   Handles: /src/behave/crown.h, /src/crown.h, /src/crown.h:L10-25"
  [uri]
  (when-let [m (re-matches #"^/src/(.+?\.(h|hpp|cpp|cc|rs|py))(?::L(\d+)(?:-(\d+))?)?$" uri)]
    (let [raw-path   (nth m 1)
          start-line (nth m 3)
          end-line   (nth m 4)]
      {:raw-path  raw-path
       :line-ref  (when start-line
                    (if end-line
                      (str start-line "-" end-line)
                      start-line))})))

;; --- HTTP handler ---

(defn- serve-static [path]
  (let [clean (str/replace path #"^/" "")
        file  (cond
                (str/starts-with? clean "src/")
                (io/file src-root (subs clean 4))

                (or (= clean "") (= clean "/"))
                (io/file project-root "index.html")

                :else
                (io/file project-root clean))]
    (if (and (.exists file) (.isFile file))
      {:status 200
       :headers {"Content-Type" (mime-type (.getName file))
                 "Cache-Control" "no-cache"}
       :body (slurp file)}
      {:status 404
       :headers {"Content-Type" "text/plain"}
       :body "Not found"})))

(defn- navigate-redirect
  "Build a redirect to index.html with a hash fragment for code navigation."
  [resolved-path line-ref]
  (let [fragment (if line-ref
                   (str "#/src/" resolved-path ":L" line-ref)
                   (str "#/src/" resolved-path))]
    {:status  302
     :headers {"Location" (str "/" fragment)}}))

(defn handler [req]
  (let [uri    (:uri req)
        method (:request-method req)]
    (cond
      ;; GET docs
      (and (= method :get) (= uri "/api/docs"))
      (let [docs (parse-org-file)]
        {:status  200
         :headers {"Content-Type"                "application/json"
                   "Access-Control-Allow-Origin" "*"}
         :body    (json/generate-string docs)})

      ;; POST parse — extract Doxygen docs from source
      (and (= method :post) (= uri "/api/parse"))
      (let [body   (json/parse-string (slurp (:body req)) true)
            folder (:folder body)
            file   (:file body)
            parsed (parse-source-file folder file)
            html   (doxygen->org-html parsed folder file)]
        {:status  200
         :headers {"Content-Type"                "application/json"
                   "Access-Control-Allow-Origin" "*"}
         :body    (json/generate-string {:parsed parsed :html html})})

      ;; POST docs
      (and (= method :post) (= uri "/api/docs"))
      (let [body (json/parse-string (slurp (:body req)))]
        (write-org-file! body)
        {:status  200
         :headers {"Content-Type"                "application/json"
                   "Access-Control-Allow-Origin" "*"}
         :body    (json/generate-string {:status "ok"})})

      ;; POST annotate/preview — compute Doxygen insertion plan
      (and (= method :post) (= uri "/api/annotate/preview"))
      (let [body   (json/parse-string (slurp (:body req)) true)
            folder (:folder body)
            file   (:file body)
            ext    (or (:ext body) ".h")]
        (try
          (let [result (preview-insertions folder file ext)]
            {:status  200
             :headers {"Content-Type"                "application/json"
                       "Access-Control-Allow-Origin" "*"}
             :body    (json/generate-string result)})
          (catch Exception e
            {:status  400
             :headers {"Content-Type"                "application/json"
                       "Access-Control-Allow-Origin" "*"}
             :body    (json/generate-string {:error (.getMessage e)})})))

      ;; POST annotate/apply — write Doxygen templates into source file
      (and (= method :post) (= uri "/api/annotate/apply"))
      (let [body       (json/parse-string (slurp (:body req)) true)
            folder     (:folder body)
            file       (:file body)
            ext        (or (:ext body) ".h")
            insertions (:insertions body)
            hash       (:hash body)]
        (try
          (let [result (apply-insertions! folder file ext insertions hash)]
            {:status  200
             :headers {"Content-Type"                "application/json"
                       "Access-Control-Allow-Origin" "*"}
             :body    (json/generate-string result)})
          (catch Exception e
            {:status  409
             :headers {"Content-Type"                "application/json"
                       "Access-Control-Allow-Origin" "*"}
             :body    (json/generate-string {:error (.getMessage e)})})))

      ;; CORS preflight
      (= method :options)
      {:status  204
       :headers {"Access-Control-Allow-Origin"  "*"
                 "Access-Control-Allow-Methods" "GET, POST, OPTIONS"
                 "Access-Control-Allow-Headers" "Content-Type"}}

      ;; /src/ navigation: browser requests get redirected to app with hash
      (and (= method :get) (str/starts-with? uri "/src/"))
      (if-let [parsed (parse-src-uri uri)]
        (let [raw       (:raw-path parsed)
              line-ref  (:line-ref parsed)
              ;; Try exact path first, then search by filename
              full-path (if (.exists (io/file src-root raw))
                          raw
                          (find-source-file (last (str/split raw #"/"))))]
          (if full-path
            ;; If request accepts HTML (browser), redirect to app
            ;; Otherwise (fetch from JS), serve the raw file
            (let [accept (get-in req [:headers "accept"] "")]
              (if (and (str/includes? accept "text/html")
                       (not (str/includes? accept "application/json")))
                (navigate-redirect full-path line-ref)
                (serve-static (str "/src/" full-path))))
            {:status 404 :headers {"Content-Type" "text/plain"} :body "Not found"}))
        (serve-static uri))

      ;; Static files
      :else
      (serve-static uri))))

;; --- Main ---

(defn -main []
  (println (str "BehavePlus Code Docs server starting on http://localhost:" port))
  (println (str "Serving files from: " (.getAbsolutePath project-root)))
  (println (str "Source root: " (.getAbsolutePath src-root)))
  (println (str "Org file: " (.getAbsolutePath org-file)))
  (http/run-server handler {:port port})
  (println "Server running. Press Ctrl+C to stop.")
  @(promise))

(-main)
