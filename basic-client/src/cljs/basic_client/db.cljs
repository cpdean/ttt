(ns basic-client.db)

(def default-db
  {
   ; tracking which side of the game you are on
   :player-name "unnamed"
   :is-connected false
   :log-text []
   :grid [[0 0 0]
          [0 0 0]
          [0 0 0]]
   :current-player-turn nil
   :winner nil
   :client-player-id nil
   })
