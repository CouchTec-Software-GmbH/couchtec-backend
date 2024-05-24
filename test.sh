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
        "interval": 60
    }
}'

curl -X POST http://localhost:3000/login \
-H "Content-Type: application/json" \
-d '{
  "username": "Linus",
  "password": "23456"
}'
