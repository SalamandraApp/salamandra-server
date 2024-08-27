import unittest
import uuid
from tests.utils import TestHelper, TestError

class TestExecutionsGetWkTemplate(unittest.TestCase):

    """
    TEST CASES
    * Incorrect path parameters
    * Not found
    * Success
    """

    def test_get_workout_execution_incorrect_path_parameters(self):
        try:
            wrong_format = TestHelper().invoke(
                    function="workout_executions", 
                    method="GET", 
                    path="/users/001/workout-executions/001",
                    path_params= {
                        "user_id": "001",
                        "workout_execution_id":"001"
                        }
                    )

            self.assertEqual(wrong_format['statusCode'], 404) 

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise
    

    def test_get_workout_execution_not_found(self):
        try:
            user_id = TestHelper().get_from_db("SELECT id FROM Users;")[0][0]
            wk_id = uuid.uuid4()
            not_found = TestHelper().invoke(
                    function="workout_executions", 
                    method="GET", 
                    path=f"/users/{user_id}/workout-executions/{wk_id}",
                    path_params= {
                        "user_id": str(user_id),
                        "workout_execution_id": str(wk_id)
                        },
                    sub=str(user_id)
                    )

            self.assertEqual(not_found['statusCode'], 404)
            self.assertEqual(not_found['body'], '"No execution exists with the corresponding id"')

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise

    def test_get_workout_execution_success(self):
        try:
            res = TestHelper().get_from_db(
               """
               SELECT WorkoutExecutions.id AS execution_id, WorkoutTemplates.user_id
               FROM WorkoutExecutions
               JOIN WorkoutTemplates ON WorkoutExecutions.workout_template_id = WorkoutTemplates.id
               LIMIT 1
               """
            )
            wk_id = res[0][0]
            user_id = res[0][1]
            success = TestHelper().invoke(
                    function="workout_executions", 
                    method="GET", 
                    path=f"/users/{user_id}/workout-executions/{wk_id}",
                    path_params= {
                        "user_id": str(user_id),
                        "workout_execution_id": str(wk_id)
                        },
                    sub=str(user_id)
                    )

            self.assertEqual(success['statusCode'], 200) 

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise



if __name__ == '__main__':
    unittest.main()
