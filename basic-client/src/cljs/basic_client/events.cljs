(ns basic-client.events
  (:require
   [re-frame.core :as re-frame]
   [basic-client.db :as db]
   ))

(re-frame/reg-event-db
 ::initialize-db
 (fn [_ _]
   db/default-db))

#_(re-frame/reg-event-db
  ::do-connection
  (fn [db [_ fn-on-event]]

    (let [url "ws://localhost:8080/"
          socket    (js/WebSocket. url)]

      (assoc db :web-socket db))))

(re-frame/reg-event-db
  :connect
  (fn [db [_]]
       (assoc db :is-connected true)))

(re-frame/reg-event-db
  :disconnect
  (fn [db [_]]
       (assoc db :is-connected false)))