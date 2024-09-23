import unittest
import uuid
from tests.utils import TestHelper, TestError

class TestExercisesGetExercise(unittest.TestCase):
    
    """
    TEST CASES
    * Wrong path parameter format   
    * Non existing exercise         
    * Existing exercise             
    """

    def test_get_exercise_incorrect_path_parameters(self):
        try:
            wrong_format = TestHelper().invoke(
                    function="exercises", 
                    method="GET", 
                    path=f"/exercises/NO-UUID",
                    path_params= {"exercise_id":"NO-UUID"}
                    )

            self.assertEqual(wrong_format['statusCode'], 404)

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise
    
    def test_get_exercise_not_found(self):
        try:
            id = str(uuid.uuid4())
            not_found = TestHelper().invoke(
                    function="exercises", 
                    method="GET", 
                    path=f"/exercises/{id}",
                    path_params= {"exercise_id": id}
                    )
            self.assertEqual(not_found['statusCode'], 404)
            self.assertEqual(not_found['body'], '"No exercise exists with the corresponding id"')

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise

    def test_get_exercise_success(self):
        try:
            id = TestHelper().get_from_db("SELECT id FROM Exercises LIMIT 1;")[0][0]
            success = TestHelper().invoke(
                    function="exercises", 
                    method="GET", 
                    path=f"/exercises/{id}",
                    path_params= {"exercise_id": id}
                    )
            self.assertEqual(success['statusCode'], 200)

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise



if __name__ == '__main__':
    unittest.main()
