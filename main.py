import logging

from app import create_app
from app.config import LOG_LEVEL, PORT

logging.basicConfig(
    level=getattr(logging, LOG_LEVEL, logging.INFO),
    format="%(levelname)s  %(name)s  %(message)s",
)
logging.getLogger("uvicorn.access").setLevel(logging.WARNING)

app = create_app()

if __name__ == "__main__":
    import uvicorn

    uvicorn.run(app, host="0.0.0.0", port=PORT)
