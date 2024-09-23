import os
import subprocess
import copy
import json
from urllib.parse import urlparse
from dotenv import load_dotenv
import psycopg2
import jwt
import string
import secrets

class TestError(Exception):
    pass

def singleton(cls):
    instances = {}
    def get_instance(*args, **kwargs):
        if cls not in instances:
            instances[cls] = cls(*args, **kwargs)
        return instances[cls]
    return get_instance

@singleton
class TestHelper:

    def get_from_db(self, command: str):
        self.cursor.execute(command)
        return self.cursor.fetchall()
        
    def invoke(
            self,
            function: str,
            method: str,
            path: str,
            query_params: dict = {},
            path_params: dict = {},
            body: dict = {},
            sub: str = ""
            ):
        if method not in self.http_methods:
            raise TestError(f"Method '{method}' is not a valid HTTP method")
        if function not in self.available_functions:
            raise TestError(f"Function '{function}' is not an available lambda function")

        event = self.__get_template()
        event["path"] = path
        event["httpMethod"] = method
        event["queryStringParameters"] = query_params
        event["multiValueQueryStringParameters"] = self.__dict_to_list(query_params)
        event["pathParameters"] = path_params
        event["body"] = json.dumps(body)
        if sub != "":
            header = {"Authorization": self.__auth_header(sub)}
            event["headers"] = header
            event["multiValueHeaders"] = self.__dict_to_list(header)

        event_ascii = json.dumps(event, ensure_ascii=True)
        command = [
                "cargo", "lambda", "invoke", 
                "-A", f"{event_ascii}", function, 
                "-o", "json", 
                "-a", "127.0.0.1", "-p", "9000"]

        result = subprocess.run(command, capture_output=True, text=True)

        if result.returncode != 0:
            raise TestError(f"Lambda invocation failed: {result.stderr}")
        return json.loads(result.stdout)

    def __init__(self):
        self.http_methods = ["GET", "HEAD", "POST", "PUT", "DELETE", "CONNECT", "OPTIONS", "TRACE", "PATCH"]
        self.available_functions = ["users", "exercises", "workout_templates", "workout_executions"]
        self.cursor = self.__initiate_sql()
        self._template = {
                "path": "",
                "httpMethod": "",
                "headers": {},
                "queryStringParameters": {},
                "multiValueQueryStringParameters": {},
                "pathParameters": {},
                "multiValueHeaders": {},
                "headers": {},
                "requestContext": {
                    "accountId": "",
                    "resourceId": "",
                    "path": "",
                    "domainName": "gy415nuibc.execute-api.us-east-2.amazonaws.com",
                    "domainPrefix": "",
                    "requestId": "",
                    "protocol": "",
                    "identity": {
                        "cognitoIdentityPoolId": "",
                        "accountId": "",
                        "cognitoIdentityId": "",
                        "caller": "",
                        "apiKey": "",
                        "apiKeyId": "",
                        "accessKey": "",
                        "sourceIp": "",
                        "cognitoAuthenticationType": "",
                        "cognitoAuthenticationProvider": "",
                        "userArn": "",
                        "userAgent": "",
                        "user": ""
                        },
                    "resourcePath": "/{proxy+}",
                    "httpMethod": "GET",
                    "requestTime": "",
                    "requestTimeEpoch": 0,
                    "apiId": "gy415nuibc"
                    },
                "body": ""
                }

    def __get_template(self):
        return copy.deepcopy(self._template)
    
    @staticmethod
    def random_string(n):
        alphabet = string.ascii_letters + string.digits
        return ''.join(secrets.choice(alphabet) for _ in range(n))

    @staticmethod
    def __auth_header(sub):
        my_claims = {
        'sub': str(sub),
        'exp': 10000000000  # Expiration timestamp
        }
        
        token = jwt.encode(my_claims, 'secret', algorithm='HS256')
        return f"Bearer {token}"


    @staticmethod
    def __dict_to_list(input_dict):
        transformed_dict = {}
        for key, value in input_dict.items():
            transformed_dict[key] = [value]
        return transformed_dict

    @staticmethod
    def __initiate_sql():
        load_dotenv()
        DATABASE_URL = os.getenv('DATABASE_URL')
        if not DATABASE_URL:
            raise ValueError("No DATABASE_URL environment variable set")

        url = urlparse(DATABASE_URL)

        dbname = url.path[1:]
        user = url.username
        password = url.password
        host = url.hostname
        port = url.port

        conn = psycopg2.connect(
            dbname=dbname,
            user=user,
            password=password,
            host=host,
            port=port
        )
        cursor = conn.cursor()
        return cursor

