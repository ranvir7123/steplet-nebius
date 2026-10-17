import unittest
from calc import add
from tests import fixtures


class TestAdd(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        fixtures.warm_cache()

    def test_add(self):
        self.assertEqual(add(2, 3), 5)
