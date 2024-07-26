import unittest
import uuid, json
from tests.utils import TestHelper, TestError

class TestExercisesGetAllWkTemplates(unittest.TestCase):

    def test_get_all_workout_templates_incorrect_path_parameters(self):
        try:
            wrong_format = TestHelper().invoke(
                    function="workout_templates", 
                    method="GET", 
                    path="/users/001/workout-templates",
                    path_params= {
                        "user_id": "001",
                        }
                    )

            self.assertEqual(wrong_format['statusCode'], 400) 

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise
    

    def test_get_all_workout_templates_success(self):
        try:
            user_id = TestHelper().get_from_db(f"SELECT user_id FROM WorkoutTemplates;")[0][0]
            success = TestHelper().invoke(
                    function="workout_templates", 
                    method="GET", 
                    path=f"/users/{user_id}/workout-templates",
                    path_params= {
                        "user_id": str(user_id),
                        },
                    sub=str(user_id)
                    )

            self.assertEqual(success['statusCode'], 200) 
            self.assertNotEqual(json.loads(success['body'])['count'], 0) 
        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise
    
if __name__ == '__main__':
    unittest.main()
