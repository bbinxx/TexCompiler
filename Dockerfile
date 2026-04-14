# Use a lightweight Python image
FROM python:3.11-slim

# Install LaTeX dependencies
RUN apt-get update && apt-get install -y \
    texlive-latex-extra \
    texlive-xetex \
    texlive-fonts-recommended \
    texlive-lang-all \
    latexmk \
    ghostscript \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy requirements and install
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy application code
COPY . .

# Expose port
EXPOSE 8000

# Run the application
CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8000"]
