from contextlib import asynccontextmanager

import asyncpg
from fastapi import Depends, FastAPI
from pydantic import BaseModel


class User(BaseModel):
    user_id: str
    email: str
    username: str


@asynccontextmanager
async def lifespan(app: FastAPI):
    async with asyncpg.create_pool(
        "postgres://postgres:password@localhost:5432/benchmark",
        min_size=5,
        max_size=5,
    ) as pool:
        app.pool = pool
        yield


app = FastAPI(lifespan=lifespan)
app.pool: asyncpg.Pool


async def db_session():
    async with app.pool.acquire() as session:
        yield session


@app.get("/", response_model=list[User])
async def read_users(session: asyncpg.Connection = Depends(db_session)):
    users = await session.fetch(
        'SELECT user_id, username, email FROM "user" ORDER BY user_id LIMIT 100'
    )
    return map(dict, users)
