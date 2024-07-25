import unittest
import json
from tests.utils import TestHelper, TestError

class TestExercisesSearchUsers(unittest.TestCase):

    def test_search_user_incorrect_query_parameters(self):
        try:
            no_parameter = TestHelper().invoke(
                    function="users", 
                    method="GET", 
                    path="/users",
                    )
            other_parameter = TestHelper().invoke(
                    function="users", 
                    method="GET", 
                    path="/users",
                    query_params={"other": "123"}
                    )

            self.assertEqual(no_parameter['statusCode'], 400)
            self.assertEqual(other_parameter['statusCode'], 400) 

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise

    def test_search_user_not_found(self):
        try:
            not_found = TestHelper().invoke(
                    function="users", 
                    method="GET", 
                    path="/users",
                    query_params={"username": "$$$"}
                    )

            self.assertEqual(not_found['statusCode'], 200) 
            self.assertEqual(len(json.loads(not_found['body'])['users']), 0) 

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise
    
    def test_search_user_success(self):
        try:
            success = TestHelper().invoke(
                    function="users", 
                    method="GET", 
                    path="/users",
                    query_params={"username": ""}
                    )

            self.assertEqual(success['statusCode'], 200) 
            self.assertNotEqual(len(json.loads(success['body'])['users']), 0) 

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise

if __name__ == '__main__':
    unittest.main()
