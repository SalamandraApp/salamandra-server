import unittest
import uuid
from datetime import datetime
from tests.utils import TestHelper, TestError

class TestExecutionsCreateWkExecution(unittest.TestCase):

    """
    TEST CASES
    * Success
    * Incorrect user id format
    * Incorrect payload
    * Invalid exercise id
    """

    def test_create_wk_execution_incorrect_path_parameters(self):
        try:
            wrong_format = TestHelper().invoke(
                    function="workout_executions", 
                    method="POST", 
                    path=f"/users/{101}/workout-executions",
                    path_params= {"user_id":"001"}
                    )

            self.assertEqual(wrong_format['statusCode'], 404) 

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise

    def test_create_wk_execution_incorrect_payload(self):
        try:
            id = TestHelper().get_from_db("SELECT id from Users;")[0][0]
            incorrect_payload = TestHelper().invoke(
                    function="workout_executions", 
                    method="POST", 
                    path=f"/users/{id}/workout-executions",
                    path_params= {"user_id":f"{id}"},
                    body={"Song" : "Visions of Dallas"},
                    sub=id,
                    )

            self.assertEqual(incorrect_payload['statusCode'], 400) 

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise

    def test_create_wk_execution_success(self):
        try:
            ex_id = TestHelper().get_from_db("SELECT id from Exercises;")[0][0]
            res = TestHelper().get_from_db("SELECT id, user_id from WorkoutTemplates;")[0]
            user_id = res[1]
            template_id = res[0]
            body = {
                "workout_template_id": str(template_id),
                "date": datetime.now().date().strftime('%Y-%m-%d'),
                "survey": 0,
                "elements": [ {
                        "exercise_id": str(ex_id),
                        "exercise_number": 1,
                        "position": 1,
                        "reps": 1,
                        "set_number": 1,
                        "weight": 1.0,
                        "rest": 0,
                        "super_set": None,
                        "time": 1
                },]
            }

            success = TestHelper().invoke(
                    function="workout_executions", 
                    method="POST", 
                    path=f"/users/{user_id}/workout-executions",
                    path_params= {"user_id": user_id},
                    body=body,
                    sub=user_id,
                    )
            print(f"BODY SUCCESS: {success['body']}")
            self.assertEqual(success['statusCode'], 201)

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise

    def test_create_wk_execution_exercise_not_found(self):
        try:
            ex_id = uuid.uuid4()
            res = TestHelper().get_from_db("SELECT id, user_id from WorkoutTemplates;")[0]
            user_id = res[1]
            template_id = res[0]
            body = {
                "workout_template_id": str(template_id),
                "date": datetime.now().date().strftime('%Y-%m-%d'),
                "survey": 0,
                "elements": [ {
                        "exercise_id": str(ex_id),
                        "exercise_number": 1,
                        "position": 1,
                        "reps": 1,
                        "set_number": 1,
                        "weight": 1.0,
                        "rest": 0,
                        "super_set": None,
                        "time": 1
                },]
            }

            not_found= TestHelper().invoke(
                    function="workout_executions", 
                    method="POST", 
                    path=f"/users/{user_id}/workout-executions",
                    path_params= {"user_id": user_id},
                    body=body,
                    sub=user_id,
                    )
            
            print(f"BODY NOT FOUND: {not_found['body']}")
            self.assertEqual(not_found['statusCode'], 404)

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise


if __name__ == '__main__':
    unittest.main()
