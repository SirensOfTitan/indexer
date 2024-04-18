(require '[babashka.http-client :as http])
(require '[clojure.java.io :as io])
(require '[babashka.fs :as fs])

(def ^:const supported-platforms
  ["macos-x86_64", "macos-aarch64", "linux-x86_64"])

(def ^:const version "v0.1.2")

(defn url-for-platform [platform]
  (str "https://github.com/asg017/sqlite-vss/releases/download/"
             version
             "/sqlite-vss-"
             version
             "-static-"
             platform
             ".tar.gz"))

(defn save-path-for-platform [platform]
  (let [current-dir (fs/parent *file*)
        file-name (str "sqlite-vss-" platform ".tar.gz")]
    (.resolve current-dir file-name)))

(defn download-file [url path]
  (io/copy
   (:body (http/get url {:as :stream}))
   (io/file (str path))))

(defn download-for-platform [platform]
  (let [url (url-for-platform platform)
        save-path (save-path-for-platform platform)]
    (download-file url save-path)))

(doseq [platform supported-platforms]
  (download-for-platform platform))
