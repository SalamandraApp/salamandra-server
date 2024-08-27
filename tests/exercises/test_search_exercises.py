import unittest
import json
from tests.utils import TestHelper, TestError

class TestExercisesSearchExercises(unittest.TestCase):

    """
    TEST CASES
    * No query parameter
    * No exercises match
    * Multiple match
    """

    def test_search_exercise_incorrect_query_parameters(self):
        try:
            no_parameter = TestHelper().invoke(
                    function="exercises", 
                    method="GET", 
                    path="/exercises",
                    )
            other_parameter = TestHelper().invoke(
                    function="exercises", 
                    method="GET", 
                    path="/exercises",
                    query_params={"other": "123"}
                    )

            self.assertEqual(no_parameter['statusCode'], 400)
            self.assertEqual(other_parameter['statusCode'], 400) 

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise

    def test_search_exercise_not_found(self):
        try:
            not_found = TestHelper().invoke(
                    function="exercises", 
                    method="GET", 
                    path="/exercises",
                    query_params={"name": "$$$"}
                    )

            self.assertEqual(not_found['statusCode'], 200) 
            self.assertEqual(len(json.loads(not_found['body'])['exercises']), 0) 

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise
    
    def test_search_exercise_success(self):
        try:
            success = TestHelper().invoke(
                    function="exercises", 
                    method="GET", 
                    path="/exercises",
                    query_params={"name": ""}
                    )

            self.assertEqual(success['statusCode'], 200)
            self.assertNotEqual(len(json.loads(success['body'])['exercises']), 0) 

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise

if __name__ == '__main__':
    unittest.main()
