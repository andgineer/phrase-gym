UPDATE current_view SET phrase_id=NULL, view_token=NULL, kind=NULL, practice=0,
  revealed=0, cue_index=NULL, hint_index=NULL, operations_json='{}'
  WHERE kind='first_study';
