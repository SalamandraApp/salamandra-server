import unittest
import uuid
from tests.utils import TestHelper, TestError

class TestExercisesGetWkTemplate(unittest.TestCase):

    def test_get_workout_template_incorrect_path_parameters(self):
        try:
            wrong_format = TestHelper().invoke(
                    function="workout_templates", 
                    method="GET", 
                    path="/users/001/workout-templates/001",
                    path_params= {
                        "user_id": "001",
                        "workout_template_id":"001"
                        }
                    )

            self.assertEqual(wrong_format['statusCode'], 400) 

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise
    

    def test_get_workout_template_not_found(self):
        try:
            user_id = TestHelper().get_from_db("SELECT id FROM Users;")[0][0]
            wk_id = uuid.uuid4()
            # wk_id = TestHelper().get_from_db(f"SELECT id FROM WorkoutTemplates WHERE user_id = {user_id};")[0][0]
            wrong_format = TestHelper().invoke(
                    function="workout_templates", 
                    method="GET", 
                    path=f"/users/{user_id}/workout-templates/{wk_id}",
                    path_params= {
                        "user_id": str(user_id),
                        "workout_template_id": str(wk_id)
                        },
                    sub=str(user_id)
                    )

            self.assertEqual(wrong_format['statusCode'], 404) 

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise

    def test_get_workout_template_success(self):
        try:
            res = TestHelper().get_from_db(f"SELECT id, user_id FROM WorkoutTemplates;")[0]
            wk_id = res[0]
            user_id = res[1]
            success = TestHelper().invoke(
                    function="workout_templates", 
                    method="GET", 
                    path=f"/users/{user_id}/workout-templates/{wk_id}",
                    path_params= {
                        "user_id": str(user_id),
                        "workout_template_id": str(wk_id)
                        },
                    sub=str(user_id)
                    )

            self.assertEqual(success['statusCode'], 200) 

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise



if __name__ == '__main__':
    unittest.main()
