"""
Security middleware and utilities for Python Worker

Implements:
- Path traversal protection
- File validation
- Request size limits
"""

import os
from pathlib import Path
from typing import Optional

import structlog
from fastapi import HTTPException, status

from app.config import settings

logger = structlog.get_logger()

# Allowed MIME types for uploads
ALLOWED_MIME_TYPES = {
    "application/pdf",
    "image/png",
    "image/jpeg",
    "image/jpg",
    "image/tiff",
    "text/plain",
}


def validate_file_path(file_path: str) -> Path:
    """
    Validate file path to prevent directory traversal attacks

    Ensures:
    1. Path is within SHARED_UPLOADS_PATH
    2. No symlink tricks
    3. File exists and is readable

    Args:
        file_path: File path to validate (can be relative or absolute)

    Returns:
        Canonicalized absolute Path

    Raises:
        HTTPException: If path is invalid or outside allowed directory
    """
    try:
        # Convert to Path and resolve to absolute canonical path
        path = Path(file_path).resolve(strict=False)

        # Get canonical upload directory
        upload_dir = Path(settings.shared_uploads_path).resolve(strict=False)

        # Check if path is within upload directory (prevents ../ attacks)
        if not path.is_relative_to(upload_dir):
            logger.warning(
                "path_traversal_attempt",
                file_path=file_path,
                resolved_path=str(path),
                upload_dir=str(upload_dir),
            )
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail="Access denied: file path outside allowed directory",
            )

        # Check file exists
        if not path.exists():
            logger.error("file_not_found", file_path=str(path))
            raise HTTPException(
                status_code=status.HTTP_404_NOT_FOUND,
                detail=f"File not found: {file_path}",
            )

        # Check it's a file (not directory)
        if not path.is_file():
            logger.error("not_a_file", file_path=str(path))
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail="Path is not a file",
            )

        # Check file is readable
        if not os.access(path, os.R_OK):
            logger.error("file_not_readable", file_path=str(path))
            raise HTTPException(
                status_code=status.HTTP_403_FORBIDDEN,
                detail="File not readable",
            )

        logger.debug("file_path_validated", file_path=str(path))
        return path

    except ValueError as e:
        # Path.resolve() can raise ValueError for invalid paths
        logger.error("invalid_file_path", file_path=file_path, error=str(e))
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=f"Invalid file path: {str(e)}",
        )


def validate_mime_type(mime_type: str) -> None:
    """
    Validate MIME type against whitelist

    Args:
        mime_type: MIME type to validate

    Raises:
        HTTPException: If MIME type not allowed
    """
    if mime_type not in ALLOWED_MIME_TYPES:
        logger.warning("unsupported_mime_type", mime_type=mime_type)
        raise HTTPException(
            status_code=status.HTTP_415_UNSUPPORTED_MEDIA_TYPE,
            detail=f"Unsupported MIME type: {mime_type}. Allowed: {', '.join(ALLOWED_MIME_TYPES)}",
        )

    logger.debug("mime_type_validated", mime_type=mime_type)


def validate_file_size(file_path: Path) -> None:
    """
    Validate file size is within limits

    Args:
        file_path: Path to file

    Raises:
        HTTPException: If file exceeds size limit
    """
    max_size_bytes = settings.max_upload_size_mb * 1024 * 1024
    file_size = file_path.stat().st_size

    if file_size > max_size_bytes:
        logger.warning(
            "file_too_large",
            file_size=file_size,
            max_size=max_size_bytes,
            file_path=str(file_path),
        )
        raise HTTPException(
            status_code=status.HTTP_413_REQUEST_ENTITY_TOO_LARGE,
            detail=f"File size {file_size / 1024 / 1024:.2f}MB exceeds limit of {settings.max_upload_size_mb}MB",
        )

    logger.debug("file_size_validated", file_size=file_size, file_path=str(file_path))


def sanitize_filename(filename: str) -> str:
    """
    Sanitize filename to prevent injection attacks

    Removes:
    - Path separators (/, \\)
    - Null bytes
    - Control characters

    Args:
        filename: Original filename

    Returns:
        Sanitized filename
    """
    # Remove path separators
    filename = filename.replace("/", "_").replace("\\", "_")

    # Remove null bytes
    filename = filename.replace("\x00", "")

    # Remove control characters (ASCII 0-31 and 127)
    filename = "".join(char for char in filename if ord(char) >= 32 and ord(char) != 127)

    # Limit length
    if len(filename) > 255:
        filename = filename[:255]

    return filename
