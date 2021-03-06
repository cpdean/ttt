(ns basic-client.events
  (:require
   [re-frame.core :as re-frame]
   [basic-client.db :as db]
   [clojure.string :as s]
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

(re-frame/reg-event-fx
  :new-ws-message
  (fn [cofx [_ event-data]]
    (let [
          forwarded-event (case (.-event_type event-data)
                            "chat" [:new-chat-message event-data]
                            "board" [:new-game-message event-data])
          ]
      {:dispatch forwarded-event})))

(re-frame/reg-event-fx
  :new-game-message
  (fn [cofx [_ event-data]]
    (let [
          content (.-content event-data)
          db (:db cofx)
          new-db (-> db
                     (assoc :grid (.-grid content))
                     (assoc :current-player-turn (.-current_player_turn content))
                     (assoc :winner (.-winner content))
                     (assoc :client-player-id (.-player_id event-data)))
          ]
      (js/console.log content)
      {:db new-db 
      })))

(re-frame/reg-event-fx
  :new-chat-message
  (fn [cofx [_ event-data]]
    (let [
          message-count (.-message-count event-data)
          content (.-content event-data)
          db (:db cofx)
          old-logs (:log-text db)
          new-logs (conj old-logs {:message-count message-count :content content})
          ]
      {:db (assoc db :log-text new-logs)
      })))


; rotating for now, irl event set later
(re-frame/reg-event-fx
  :move
  (fn [cofx [_ pos]]
    (let [
          db (:db cofx)
          [x y] pos
          ]
       {
        :dispatch [:send-move pos]
        })))

(re-frame/reg-event-fx
  :send-move
  (fn [cofx [_ [x y]]]
    (let [
          json js/JSON.stringify
          move-event (json #js {:position #js [x y]})
          event-str (json #js {:event_type "move" :data move-event})
          ]
      {:send-event event-str}
      )))

(re-frame/reg-event-fx
  :send-msg
  (fn [cofx [_ text-msg]]
    (let [
          json js/JSON.stringify
          event-str (json #js {:event_type "chatmessage" :data text-msg})
          ]
      {:send-event event-str}
      )))

(re-frame/reg-fx
  :send-event
  (fn [event-str]
    (let [ socket (.-tttconn js/window) ]
      (.send socket event-str ))))
