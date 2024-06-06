curl -X GET http://localhost:3000/projects/20 \
-H "Content-Type: application/json"

curl -X PUT http://localhost:3000/projects/20 \
-H "Content-Type: application/json" \
-d '{
    "name": "Linus",
    "price": 100,
    "description": "This is a test",
    "monitoring": {
        "enabled": true,
        "interval": 64200
    }
}'

curl -X POST http://localhost/api/pre-register \
-H "Content-Type: application/json" \
-d '{
  "email": "linus@couchtec.com",
  "password": "23456",
  "newsletter": false
}'
curl -X POST http://localhost/api/register \
-H "Content-Type: application/json" \
-d '{
  "uuid": "9e1bff29-5772-4a87-86c0-18277344c990"
}'

curl -X POST http://localhost/api/login \
-H "Content-Type: application/json" \
-d '{
  "email": "linus@couchtec.com",
  "password": "23456"
}'

curl -X POST http://localhost/api/pre-reset \
-H "Content-Type: application/json" \
-d '{
  "email": "linus@couchtec.com"
}'

curl -X POST http://localhost/api/reset \
-H "Content-Type: application/json" \
-d '{
 "uuid": "53a383c1-e575-4a42-95e9-51cc1cf291b3",
  "password": "lol"
}'

curl -X GET http://localhost:3000/uuids/Linus \
-H "Content-Type: application/json"

curl -X PUT http://localhost:3000/uuids/Linus \
-H "Content-Type: application/json" \
-d '["12234", "5678"]'

curl -X POST http://localhost:80/reset \
-H "Content-Type: application/json" \
-d '{
  "email": "linus@couchtec.com"
}'


