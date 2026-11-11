UPDATE imports SET error='This package was saved without reference audio, '
  || 'which Phrase Gym no longer prepares for packages. Its content is kept. Ask your AI agent '
  || 'to submit it again with complete audio under a new package_id.'
  WHERE kind='package' AND status='error';
