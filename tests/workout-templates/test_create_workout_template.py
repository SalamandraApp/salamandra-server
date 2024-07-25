import unittest
import uuid
from datetime import datetime
from tests.utils import TestHelper, TestError

class TestExercisesCreateWkTemplate(unittest.TestCase):

    def test_create_wk_template_incorrect_path_parameters(self):
        try:
            wrong_format = TestHelper().invoke(
                    function="workout_templates", 
                    method="POST", 
                    path=f"/users/{101}/workout-templates",
                    path_params= {"user_id":"001"}
                    )

            self.assertEqual(wrong_format['statusCode'], 400) 

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise
    def test_create_wk_template_incorrect_payload(self):
        try:
            id = TestHelper().get_from_db("SELECT id from Users;")[0][0]
            incorrect_payload = TestHelper().invoke(
                    function="workout_templates", 
                    method="POST", 
                    path=f"/users/{id}/workout-templates",
                    path_params= {"user_id":f"{id}"},
                    body={"Song" : "Visions of Dallas"},
                    sub=id,
                    )

            self.assertEqual(incorrect_payload['statusCode'], 400) 

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise

    def test_create_wk_template_success(self):
        try:
            ex_id = TestHelper().get_from_db("SELECT id from Exercises;")[0][0]
            id = TestHelper().get_from_db("SELECT id from Users;")[0][0]
            body = {
                "name": "W1",
                "description": "",
                "date_created": datetime.now().date().strftime('%Y-%m-%d'),
                "elements": [ {
                        "exercise_id": str(ex_id),  # Generates a unique UUID
                        "position": 1,
                        "reps": 1,
                        "sets": 1,
                        "weight": 0.0,
                        "rest": 0,
                        "super_set": None
                },]
            }

            not_found = TestHelper().invoke(
                    function="workout_templates", 
                    method="POST", 
                    path=f"/users/{id}/workout-templates",
                    path_params= {"user_id": id},
                    body=body,
                    sub=id,
                    )
            print(not_found['body'])
            self.assertEqual(not_found['statusCode'], 201)

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise

    def test_create_wk_template_exercise_not_found(self):
        try:
            ex_id = uuid.uuid4()
            id = TestHelper().get_from_db("SELECT id from Users;")[0][0]
            body = {
                "name": "W1",
                "description": "",
                "date_created": datetime.now().date().strftime('%Y-%m-%d'),
                "elements": [ {
                        "exercise_id": str(ex_id), 
                        "position": 1,
                        "reps": 1,
                        "sets": 1,
                        "weight": 0.0,
                        "rest": 0,
                        "super_set": None
                },]
            }

            success = TestHelper().invoke(
                    function="workout_templates", 
                    method="POST", 
                    path=f"/users/{id}/workout-templates",
                    path_params= {"user_id": id},
                    body=body,
                    sub=id,
                    )
            print(success['body'])
            self.assertEqual(success['statusCode'], 404)

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise



if __name__ == '__main__':
    unittest.main()
