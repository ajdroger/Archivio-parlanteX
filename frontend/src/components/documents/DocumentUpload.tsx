import { useState, useRef } from 'react';
import { Upload, FileText, Loader2, CheckCircle, XCircle } from 'lucide-react';
import api from '../../lib/api';

interface DocumentUploadProps {
  kbId: string;
  onUploadComplete: () => void;
}

interface UploadingFile {
  file: File;
  status: 'uploading' | 'success' | 'error';
  error?: string;
}

/**
 * Document upload component with drag-and-drop support.
 * Allows users to upload PDF, DOCX, and TXT files to a knowledge base.
 */
export default function DocumentUpload({
  kbId,
  onUploadComplete,
}: DocumentUploadProps) {
  const [isDragging, setIsDragging] = useState(false);
  const [uploadingFiles, setUploadingFiles] = useState<UploadingFile[]>([]);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);

    const files = Array.from(e.dataTransfer.files);
    handleFiles(files);
  };

  const handleFileInput = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files) {
      const files = Array.from(e.target.files);
      handleFiles(files);
    }
  };

  const handleFiles = async (files: File[]) => {
    // Filter accepted file types
    const acceptedTypes = [
      'application/pdf',
      'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
      'text/plain',
    ];

    const validFiles = files.filter((file) => {
      if (!acceptedTypes.includes(file.type)) {
        alert(`File ${file.name} non supportato. Solo PDF, DOCX e TXT.`);
        return false;
      }
      // Max 200MB per file (from CLAUDE.md)
      const maxSize = 200 * 1024 * 1024;
      if (file.size > maxSize) {
        alert(`File ${file.name} troppo grande. Massimo 200MB.`);
        return false;
      }
      return true;
    });

    if (validFiles.length === 0) return;

    // Initialize uploading state
    const newUploading: UploadingFile[] = validFiles.map((file) => ({
      file,
      status: 'uploading',
    }));
    setUploadingFiles((prev) => [...prev, ...newUploading]);

    // Upload files sequentially
    for (let i = 0; i < validFiles.length; i++) {
      const file = validFiles[i];
      try {
        await api.uploadDocument(kbId, file);
        setUploadingFiles((prev) =>
          prev.map((uf) =>
            uf.file === file ? { ...uf, status: 'success' } : uf
          )
        );
      } catch (err: unknown) {
        const errorMessage = err instanceof Error ? err.message : 'Errore durante il caricamento';
        setUploadingFiles((prev) =>
          prev.map((uf) =>
            uf.file === file
              ? { ...uf, status: 'error', error: errorMessage }
              : uf
          )
        );
      }
    }

    // Clear file input
    if (fileInputRef.current) {
      fileInputRef.current.value = '';
    }

    // Notify parent component
    onUploadComplete();
  };

  const clearCompleted = () => {
    setUploadingFiles((prev) =>
      prev.filter((uf) => uf.status === 'uploading')
    );
  };

  const totalSize = uploadingFiles.reduce((sum, uf) => sum + uf.file.size, 0);
  const successCount = uploadingFiles.filter((uf) => uf.status === 'success').length;
  const errorCount = uploadingFiles.filter((uf) => uf.status === 'error').length;
  const uploadingCount = uploadingFiles.filter((uf) => uf.status === 'uploading').length;

  return (
    <div className="space-y-4">
      {/* Drag and drop zone */}
      <div
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
        onClick={() => fileInputRef.current?.click()}
        className={`relative border-2 border-dashed rounded-lg p-8 text-center cursor-pointer transition-colors ${
          isDragging
            ? 'border-primary bg-primary/10'
            : 'border-dark-50 hover:border-primary/50 hover:bg-dark-50'
        }`}
      >
        <input
          ref={fileInputRef}
          type="file"
          multiple
          accept=".pdf,.docx,.txt"
          onChange={handleFileInput}
          className="hidden"
        />

        <Upload
          className={`w-12 h-12 mx-auto mb-4 ${
            isDragging ? 'text-primary' : 'text-gray-400'
          }`}
        />
        <p className="text-sm font-medium text-gray-300 mb-1">
          Trascina file qui o clicca per selezionare
        </p>
        <p className="text-xs text-gray-500">
          PDF, DOCX, TXT • Massimo 200MB per file
        </p>
      </div>

      {/* Uploading files list */}
      {uploadingFiles.length > 0 && (
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <h4 className="text-sm font-medium text-gray-300">
              Upload ({uploadingFiles.length} file)
            </h4>
            {(successCount > 0 || errorCount > 0) && uploadingCount === 0 && (
              <button
                onClick={clearCompleted}
                className="text-xs text-gray-500 hover:text-gray-400"
              >
                Cancella completati
              </button>
            )}
          </div>

          <div className="space-y-2">
            {uploadingFiles.map((uf, idx) => (
              <div
                key={idx}
                className="flex items-center gap-3 px-3 py-2 bg-dark-100 border border-dark-50 rounded-lg"
              >
                {/* Status icon */}
                {uf.status === 'uploading' && (
                  <Loader2 className="w-4 h-4 text-primary animate-spin flex-shrink-0" />
                )}
                {uf.status === 'success' && (
                  <CheckCircle className="w-4 h-4 text-green-400 flex-shrink-0" />
                )}
                {uf.status === 'error' && (
                  <XCircle className="w-4 h-4 text-red-400 flex-shrink-0" />
                )}

                {/* File info */}
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <FileText className="w-4 h-4 text-gray-400 flex-shrink-0" />
                    <span className="text-sm text-gray-300 truncate">
                      {uf.file.name}
                    </span>
                  </div>
                  {uf.error && (
                    <p className="text-xs text-red-400 mt-1">{uf.error}</p>
                  )}
                </div>

                {/* File size */}
                <span className="text-xs text-gray-500 flex-shrink-0">
                  {(uf.file.size / 1024).toFixed(1)} KB
                </span>
              </div>
            ))}
          </div>

          {/* Summary */}
          <div className="flex items-center gap-4 text-xs text-gray-500">
            <span>Totale: {(totalSize / 1024 / 1024).toFixed(2)} MB</span>
            {successCount > 0 && (
              <span className="text-green-400">✓ {successCount} completati</span>
            )}
            {errorCount > 0 && (
              <span className="text-red-400">✗ {errorCount} errori</span>
            )}
            {uploadingCount > 0 && (
              <span className="text-primary">↑ {uploadingCount} in corso</span>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
