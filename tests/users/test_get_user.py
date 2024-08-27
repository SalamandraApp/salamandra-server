import unittest
import uuid
from tests.utils import TestHelper, TestError

class TestUsersGetUser(unittest.TestCase):

    """
    TEST CASES
    * Wrong path parameter format   
    * Non existing exercise         
    * Existing exercise             
    """

    def test_get_user_incorrect_path_parameters(self):
        try:
            wrong_format = TestHelper().invoke(
                    function="users", 
                    method="GET", 
                    path="/users/NO-UUID",
                    path_params= {"user_id":"NO-UUID"}
                    )

            self.assertEqual(wrong_format['statusCode'], 404) 

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise
    
    def test_get_user_not_found(self):
        try:
            id = str(uuid.uuid4())
            not_found = TestHelper().invoke(
                    function="users", 
                    method="GET", 
                    path=f"/users/{id}",
                    path_params= {"user_id": id}
                    )
            self.assertEqual(not_found['statusCode'], 404)
            self.assertEqual(not_found['body'], '"No user exists with the corresponding id"')

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise

    def test_get_user_success(self):
        try:
            id = TestHelper().get_from_db("SELECT id from Users;")[0][0]
            success = TestHelper().invoke(
                    function="users", 
                    method="GET", 
                    path=f"/users/{id}",
                    path_params= {"user_id": id}
                    )
            print(f"BODY: {success['body']}")
            self.assertEqual(success['statusCode'], 200)

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise

if __name__ == '__main__':
    unittest.main()
