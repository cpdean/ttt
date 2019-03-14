(ns basic-client.subs
  (:require
   [re-frame.core :as re-frame]))

(re-frame/reg-sub
 ::name
 (fn [db]
   (:name db)))

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
