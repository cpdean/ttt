(ns basic-client.views
  (:require
   [re-frame.core :as re-frame]
   [basic-client.subs :as subs]
   [clojure.pprint :refer [pprint]]
   ))

(defn disconnect
  []
  (if (.-tttconn js/window)
    (do
      (.close (.-tttconn js/window))
      (set! (.-tttconn js/window) nil)
      (re-frame/dispatch [:disconnect]))))

(defn connect
  []
  (let [url "ws://localhost:8080/ws/"
        socket    (js/WebSocket. url)]
      (set! (.-onopen socket) (fn [] (re-frame/dispatch [:connect])))
      (set! (.-onmessage socket) (fn [e] (re-frame/dispatch [:new-ws-message (.parse js/JSON (.-data e))])))
      (set! (.-onclose socket) (fn [e] (re-frame/dispatch [:disconnect])))
      ; put it on the window obj so you can close it
      (set! (.-tttconn js/window) socket)
      ))


(defn connection-panel []
  (let [is-connected (re-frame/subscribe [:is-connected])]
    (if @is-connected
      [:div
       [:button#connect
        {:on-click #(disconnect)}
        "Disconnect"] 
       [:span.spacer {:style {:padding "0em 1em"}} "|" ]
       [:span#status "currently connected" ]
       ]
      [:div
       [:button#connect
        {:on-click #(connect)}
        "Connect"] 
       [:span.spacer {:style {:padding "0em 1em"}} "|" ]
       [:span#status "disconnected" ]
       ]
      )
    ))

(defn last-n [n coll]
  (let [drop-n (- (count coll) n)]
    (drop drop-n coll)))


(defn chat-logs []
  (let [
        logs @(re-frame/subscribe [::subs/logs])
        ]
    [:div#log
     [:ul
      (for [item (last-n 5 logs)]
        (let [ct (:message-count item)
              txt (:content item)]
        ^{:key (:message-count item)} [:li "messages: " ct " " txt])) ] ]))


(defn cell [value pos]
  (js/console.log "the value")
  (js/console.log value)
  (let [
        clicker (case value
                  1 #()
                  2 #()
                  0 #(do (re-frame/dispatch [:move pos]) false))
        filler (case value 1 "X" 2 "O" 0 " ")
        style-class (case value
                      1 ""
                      2 ""
                      0 "open")
        
        ]
    [:td {:on-click clicker :class style-class} filler ]
    )
  )

(defn game-board []
  (let [
        grid @(re-frame/subscribe [::subs/grid])
        [[c00 c10 c20]
         [c01 c11 c21]
         [c02 c12 c22]] grid]
    [:div
    #_[:pre
      (with-out-str
        (pprint
          grid))] 
     [:table#gameboard
      [:tbody
       [:tr [cell c00 [0 0]] [cell c10 [1 0]] [cell c20 [2 0]]]
       [:tr [cell c01 [0 1]] [cell c11 [1 1]] [cell c21 [2 1]]]
       [:tr [cell c02 [0 2]] [cell c12 [1 2]] [cell c22 [2 2]]]
       ]
      ]]))

(defn player-status []
  (let [
        name @(re-frame/subscribe [::subs/player-name])
        ]
    [:span name ]))

(defn main-panel []
  [:div
   [player-status]
   [connection-panel]
   [game-board]
   [chat-logs]
   [:input#text {:type "text"
                 :on-key-up (fn [e]
                              (if (= (.-keyCode e) 13)
                                (let [text-input (.getElementById js/document "text")
                                      text (.-value text-input) ]

                                  (re-frame/dispatch [:send-msg text])
                                  (set! (.-value text-input) "")
                                  false)))
                 } ]
   [:input#send {
                 :type "button"
                 :value "Send"
                 :on-click (fn []
                             (let [text-input (.getElementById js/document "text")
                                   text (.-value text-input) ]
                               (re-frame/dispatch [:send-msg text])
                               (set! (.-value text-input) "")
                               false
                               ))
                 } ]


   ])
