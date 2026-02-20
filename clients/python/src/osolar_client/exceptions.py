from __future__ import annotations

from typing import Any


class ApiError(Exception):
    def __init__(self, status_code: int, message: str, response_body: Any):
        super().__init__(f"OSOLAR API error {status_code}: {message}")
        self.status_code = status_code
        self.response_body = response_body
