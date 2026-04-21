"""
Configuration for Python AI Worker

Uses pydantic-settings for environment variable parsing.
"""

from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    """Application settings"""

    # App
    app_env: str = "dev"
    app_debug: bool = True

    # API
    rust_engine_url: str = "http://rust-engine:8090"
    rust_engine_internal_token: str = ""

    # Shared storage
    shared_uploads_path: str = "/shared/uploads"
    max_upload_size_mb: int = 200

    # OCR
    tesseract_lang: str = "ita+eng"
    ocr_dpi: int = 300

    # Parsing
    pdf_max_pages: int = 500
    chunk_size: int = 512
    chunk_overlap: int = 50

    model_config = SettingsConfigDict(
        env_file=".env",
        env_file_encoding="utf-8",
        case_sensitive=False,
    )


# Global settings instance
settings = Settings()
