openssl genrsa -out ca.key 2048
openssl req -x509 -new -nodes -key ca.key -sha256 -days 360001 -out ca.crt -subj "/C=AU/ST=Victoria/L=Melbourne/O=Real CA Pty Ltd/OU=Testing/CN=ca.example.com"

openssl genrsa -out server.key 2048
openssl req -new -key server.key -out server.csr -subj "/C=AU/ST=Victoria/L=Melbourne/O=Real Server Pty Ltd/OU=Testing/CN=server.example.com"
openssl x509 -req -in server.csr -CA ca.crt -CAkey ca.key -CAcreateserial -out server.crt -days 360000 -sha256

rm ca.srl
rm server.csr
