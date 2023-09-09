from sqlalchemy.orm import Session

from app import models, schemas

def get_users(db: Session):
    return db.query(models.User).order_by(models.User.user_id).limit(100).all()
