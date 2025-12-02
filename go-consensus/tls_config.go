package main

import (
    "os"
)

func TLSCertFiles() (cert, key string, ok bool) {
    cert = os.Getenv("TLS_CERT_FILE")
    key = os.Getenv("TLS_KEY_FILE")
    if cert != "" && key != "" {
        if _, err := os.Stat(cert); err == nil {
            if _, err2 := os.Stat(key); err2 == nil {
                return cert, key, true
            }
        }
    }
    // fallback to local files
    cert = "tls/server.crt"
    key = "tls/server.key"
    if _, err := os.Stat(cert); err == nil {
        if _, err2 := os.Stat(key); err2 == nil {
            return cert, key, true
        }
    }
    return "", "", false
}
