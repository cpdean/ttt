(ns basic-client.views
  (:require
   [re-frame.core :as re-frame]
   [basic-client.subs :as subs]
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


(defn chat-logs []
  (let [
        logs @(re-frame/subscribe [::subs/logs])
        ]
    [:div#log
     [:ul
      (for [item logs]
        (let [ct (:message-count item)
              txt (:content item)]
        ^{:key (:message-count item)} [:li "messages: " ct " " txt]
        )
        )
      ]
     ]
    )
  )

(defn main-panel []
  (let [
        name (re-frame/subscribe [::subs/name])
        ]
    [:div
     [:h2 "doing aa chat in " @name]
     [connection-panel]
     [chat-logs]
     [:form#chatform {:on-submit #(false)}
      [:input#text {:type "text"} ]
      [:input#send {:type "button" :value "Send"} ]
      ]


; <div>
; </div>
; <div id="log"
;      style="width:20em;height:15em;overflow:auto;border:1px solid black">
; </div>
; <form id="chatform" onsubmit="return false;">
;   <input id="text" type="text" />
;   <input id="send" type="button" value="Send" />
; </form>



     ]))
