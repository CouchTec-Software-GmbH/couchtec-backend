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
  "uuid": "e4705207-80a3-4f14-8a8c-5e198db9fe26"
}'

curl -X POST http://localhost/api/login \
-H "Content-Type: application/json" \
-d '{
  "email": "linus@couchtec.com",
  "password": "lol"
}' 

export SESSION_TOKEN=$(curl -X POST http://localhost/api/login \
-H "Content-Type: application/json" \
-d '{
  "email": "linus@couchtec.com",
  "password": "lol"
}' | sed 's/"//g')

curl -X GET http://localhost/api/config \
-H "Content-Type: application/json" \
-H "Authorization: Bearer $SESSION_TOKEN"


curl -X GET http://localhost/api/user/last-uuid \
-H "Content-Type: application/json" \
-H "Authorization: Bearer $SESSION_TOKEN"

curl -X DELETE http://localhost/api/user/linus@couchtec.com \
-H "Content-Type: application/json" \
-H "Authorization: Bearer $SESSION_TOKEN"

curl -X POST http://localhost/api/logout?id=linus@couchtec.com \
-H "Content-Type: application/json" \
-H "Authorization: Bearer $SESSION_TOKEN"

curl -X GET http://localhost/api/uuids/linus@couchtec.com \
-H "Content-Type: application/json" \
-H "Authorization: Bearer $SESSION_TOKEN"


curl -X PUT http://localhost/api/test.6aa5280a-e007-4ed5-87fb-3e2a78db1c52 \
-H "Content-Type: application/json" \
-H "Authorization: Bearer $SESSION_TOKEN" \
-d '{
    "name": "Linus"
}'

curl -X GET http://localhost/api/20 \
-H "Content-Type: application/json" \
-H "Authorization: Bearer $SESSION_TOKEN"

curl -X GET http://localhost/api/20 \
-H "Content-Type: application/json" \
-H "Authorization: Bearer $SESSION_TOKEN"

curl -X DELETE http://localhost/api/uuids/linus@couchtec.com/20

curl -X POST http://localhost/api/uuids/linus@couchtec.com \
-d '{
  "email": "linus@couchtec.com",
  "password": "23456",
  "newsletter": false
}'

curl -X POST http://4.185.30.170:3000/pre-register \
-H "Content-Type: application/json" \
-d '{
  "email": "linus@couchtec.com",
  "password": "23456",
  "newsletter": false
}'
curl -X POST http://localhost/api/register \
-H "Content-Type: application/json" \
-H "Authorization: Bearer $SESSION_TOKEN" \
-d '{
  "uuid": "30"
}'

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

curl -X GET https://couchdb-app-service.azurewebsites.net/users/linus@couchtec.com \
-u "admin:8RzuxhQ7"
