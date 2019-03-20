(ns basic-client.subs
  (:require
   [re-frame.core :as re-frame]))

(re-frame/reg-sub
 ::name
 (fn [db]
   (:name db)))

(re-frame/reg-sub
 ::player-name
 (fn [db]
   (:player-name db)))

(re-frame/reg-sub
 ::current-player-turn
 (fn [db]
   (:current-player-turn db)))

(re-frame/reg-sub
 ::winner
 (fn [db]
   (:winner db)))

(re-frame/reg-sub
 ::client-player-id
 (fn [db]
   (:client-player-id db)))

(re-frame/reg-sub
  :is-connected
  (fn [db]
    (:is-connected db)))

(re-frame/reg-sub
  ::logs
  (fn [db]
    (:log-text db)))

(re-frame/reg-sub
  ::grid
  (fn [db]
    (:grid db)))
