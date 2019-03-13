(ns basic-client.core
  (:require
   [reagent.core :as reagent]
   [re-frame.core :as re-frame]
   [basic-client.events :as events]
   [basic-client.views :as views]
   [basic-client.config :as config]
   ))


(defn dev-setup []
  (when config/debug?
    (enable-console-print!)
    (println "dev mode")))

(defn mount-root []
  (re-frame/clear-subscription-cache!)
  (reagent/render [views/main-panel]
                  (.getElementById js/document "app")))

(defn ^:export init []
  (re-frame/dispatch-sync [::events/initialize-db])
  (dev-setup)
  (mount-root))
