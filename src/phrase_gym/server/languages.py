from typing import Literal, get_args

LearningLanguage = Literal["en", "de"]
NativeLanguage = Literal["ru", "en"]
# The meaning layer is in the native language or in the learning language itself.
MeaningLanguage = Literal["en", "de", "ru"]
LEARNING_LANGUAGES: tuple[LearningLanguage, ...] = get_args(LearningLanguage)
