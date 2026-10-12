"""Version-one HTTP and manual-package contracts."""

from typing import Annotated, Literal

from pydantic import BaseModel, ConfigDict, Field

Language = Literal["en", "ru", "sr", "de"]
Identifier = Annotated[str, Field(min_length=1, max_length=160, pattern=r"^[\w.:-]+$")]
Text = Annotated[str, Field(min_length=1, max_length=10000)]


class Contract(BaseModel):
    model_config = ConfigDict(extra="forbid")
