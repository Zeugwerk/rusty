{test}
FUNCTION myTest : DINT
    myTest := ASSERT_EQ(1, 2);
END_FUNCTION

{test}
FUNCTION myTest2 : DINT
    myTest2 := ASSERT_EQ(1, 1);
END_FUNCTION
