import unittest
import uuid
from datetime import datetime
from tests.utils import TestHelper, TestError

class TestExercisesCreateUsers(unittest.TestCase):
    
    def test_create_user_incorrect_payload(self):
        try:
            id = str(uuid.uuid4())
            body = {
                    "uuid" : id,
                    "username": TestHelper().random_string(10),
                    # "date_joined": datetime.now().date().strftime('%Y-%m-%d')
                    }
            success = TestHelper().invoke(
                    function="users", 
                    method="POST", 
                    path="/users",
                    sub=id,
                    body=body
                    )
            self.assertEqual(success['statusCode'], 400)

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise

    def test_create_user_created(self):
        try:
            id = str(uuid.uuid4())
            body = {
                    "uuid" : id,
                    "username": TestHelper().random_string(10),
                    "date_joined": datetime.now().date().strftime('%Y-%m-%d')
                    }
            success = TestHelper().invoke(
                    function="users", 
                    method="POST", 
                    path="/users",
                    sub=id,
                    body=body
                    )
            self.assertEqual(success['statusCode'], 201)

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise

    def test_create_user_conflict(self):
        try:
            id = str(uuid.uuid4())
            body = {
                    "uuid" : id,
                    "username": TestHelper().random_string(10),
                    "date_joined": datetime.now().date().strftime('%Y-%m-%d')
                    }
            success = TestHelper().invoke(
                    function="users", 
                    method="POST", 
                    path="/users",
                    sub=id,
                    body=body
                    )
            conflict = TestHelper().invoke(
                    function="users", 
                    method="POST", 
                    path="/users",
                    sub=id,
                    body=body
                    )
            self.assertEqual(success['statusCode'], 201)
            self.assertEqual(conflict['statusCode'], 409)

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise

    def test_create_user_unauthorized(self):
        try:
            id = str(uuid.uuid4())
            body = {
                    "uuid" : id,
                    "username": TestHelper().random_string(10),
                    "date_joined": datetime.now().date().strftime('%Y-%m-%d')
                    }
            unauth = TestHelper().invoke(
                    function="users", 
                    method="POST", 
                    path="/users",
                    body=body
                    )

            self.assertEqual(unauth['statusCode'], 401)

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise
    
    def test_create_user_forbidden(self):
        try:
            body = {
                    "uuid" : str(uuid.uuid4()),
                    "username": TestHelper().random_string(10),
                    "date_joined": datetime.now().date().strftime('%Y-%m-%d')
                    }
            forbidden = TestHelper().invoke(
                    function="users", 
                    method="POST", 
                    path="/users",
                    body=body,
                    sub=str(uuid.uuid4())
                    )

            self.assertEqual(forbidden['statusCode'], 403)

        except TestError as e:
            print(f"TEST ERROR: {e}")
            raise


if __name__ == '__main__':
    unittest.main()
