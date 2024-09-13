import unittest
import uuid
from tests.utils import TestHelper, TestError

class TestUsersPatchUser(unittest.TestCase):

    """
    TEST CASES
    * Wrong path parameter
    * Wrong payload
    * New user
    * Success
    """

    def test_patch_user_wrong_path_parameter(self):
        try:
            id = TestHelper().get_from_db("SELECT id from Users;")[0][0]
            body = {
                    "display_name": "NEW NAME",
                    }
            new_user = TestHelper().invoke(
                    function="users", 
                    method="PATCH", 
                    path=f"/users/not-uuid",
                    sub =str(id),
                    path_params= {"user_id": "not-uuid"},
                    body=body
                    )
            
            self.assertEqual(new_user['statusCode'], 404)

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise
    def test_patch_user_new_user(self):
        try:
            id = uuid.uuid4()
            body = {
                    "display_name": "NEW NAME",
                    }
            new_user = TestHelper().invoke(
                    function="users", 
                    method="PATCH", 
                    path=f"/users/{id}",
                    sub =str(id),
                    path_params= {"user_id": str(id)},
                    body=body
                    )
            
            self.assertEqual(new_user['statusCode'], 404)

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise
    def test_patch_user_success(self):
        try:
            id = TestHelper().get_from_db("SELECT id from Users;")[0][0]
            body = {
                    "display_name": "NEW",
                    }
            success = TestHelper().invoke(
                    function="users", 
                    method="PATCH", 
                    path=f"/users/{id}",
                    sub =id,
                    path_params= {"user_id": id},
                    body=body
                    )
            
            self.assertEqual(success['statusCode'], 200)

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise

if __name__ == '__main__':
    unittest.main()
