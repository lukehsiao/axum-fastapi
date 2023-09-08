from pydantic import BaseModel


class UserBase(BaseModel):
    email: str
    username: str

class User(UserBase):
    user_id: str

    class Config:
        from_attributes = True

