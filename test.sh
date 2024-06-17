curl -X GET http://localhost/api/bf8160da-0639-48ad-9138-bcb988199a66 \
-H "Content-Type: application/json" \
-H "Authorization: Bearer a2838716-7f7d-4433-a1ff-6ffe8a461a29"


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

curl -X GET http://localhost:3000/uuids/linus@couchtec.com

curl -X DELETE http://localhost/api/uuids/linus@couchtec.com/lol 

curl -X DELETE http://localhost/api/user/linus@couchtec.com

curl -X POST http://localhost/api/login \
-H "Content-Type: application/json" \
-d '{
  "email": "linus@couchtec.com",
  "password": "lol"
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

curl -X POST http://localhost/api/uuids/linus@couchtec.com \
-H "Content-Type: application/json" \
-d '{
  "uuid": "eyy"
}'

curl -X DELETE https://couchdb-app-service.azurewebsites.net/users/Getthemlol@protonmail.com?rev=31-c3eb40d582f71055b36d2b51b7e0fc04 \
-u "admin:8RzuxhQ7"

curl -X GET https://couchdb-app-service.azurewebsites.net/users/Getthemlol@protonmail.com \
-u "admin:8RzuxhQ7"
