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

curl -X POST http://localhost:3000/register \
-H "Content-Type: application/json" \
-d '{
  "email": "Linus",
  "password": "23456"
}'

curl -X POST http://localhost:3000/login \
-H "Content-Type: application/json" \
-d '{
  "email": "Linus",
  "password": "23456"
}'

curl -X GET http://localhost:3000/uuids/Linus \
-H "Content-Type: application/json"

curl -X PUT http://localhost:3000/uuids/Linus \
-H "Content-Type: application/json" \
-d '["12234", "5678"]'
