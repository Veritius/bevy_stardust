openssl x509 -noout -modulus -in ca.crt | openssl sha256
openssl rsa -noout -modulus -in ca.key | openssl sha256

openssl x509 -noout -modulus -in server.crt | openssl sha256
openssl rsa -noout -modulus -in server.key | openssl sha256