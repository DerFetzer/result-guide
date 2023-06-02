# `result-guide`
Your advanced test management.

## Note
This is an educational project accompanying [in-code-we-rust](https://github.com/DerFetzer/in-code-we-rust) exercises.

curl -d '{"date":"2014-11-28T21:00:09+09:00","project":"TEST-PROJECT","name":"MyFancyTestCase.pkg","verdict":"SUCCESS"}' http://localhost:3000/reports
curl -d '{"date":"2015-11-28T21:00:09+09:00","project":"TEST-PROJECT","name":"MyFancyTestCase_new.pkg","verdict":"FAILED"}' http://localhost:3000/reports

curl -H "Content-Type: application/json" -d '{"name": "Bus Lesen", "step_number": 1, "date": "2014-11-28T21:00:01+09:00", "verdict": "NONE"}' http://localhost:3000/reports/1/test_steps     
curl -H "Content-Type: application/json" -d '{"name": "Mess Lesen", "step_number": 2, "date": "2014-11-28T21:00:02+09:00", "verdict": "SUCCESS"}' http://localhost:3000/reports/1/test_steps 

curl -H "Content-Type: application/json" -d '{"name": "Kalib Lesen", "step_number": 1, "date": "2013-11-28T21:00:02+09:00", "verdict": "FAILED"}' http://localhost:3000/reports/2/test_steps 
