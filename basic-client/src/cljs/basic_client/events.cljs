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

(re-frame/reg-event-db
  :new-ws-message
  (fn [db [_ event-data]]
    (let [
          message-count (.-message-count event-data)
          content (.-content event-data)
          old-logs (:log-text db)
          new-logs (conj old-logs {:message-count message-count :content content})]
       (assoc db :log-text new-logs))))


; rotating for now, irl event set later
(re-frame/reg-event-db
  :move
  (fn [db [_ pos]]
    (let [[x y] pos
          _ (js/console.log pos)
          current (((:grid db) y) x)
          rot-cell (case current
                     0 1
                     1 2
                     2 0)
          new-row (assoc ((:grid db) y) x rot-cell)
          new-grid (assoc (:grid db) y new-row)]
       (assoc db :grid new-grid))))
