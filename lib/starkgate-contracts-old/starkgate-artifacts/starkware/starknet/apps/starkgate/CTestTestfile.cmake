# CMake generated Testfile for 
# Source directory: /app/src/starkware/starknet/apps/starkgate
# Build directory: /app/build/Release/src/starkware/starknet/apps/starkgate
# 
# This file includes the relevant testing commands required for 
# testing this directory and lists subdirectories to be tested as well.
add_test([=[starkgate_flow_test]=] "/app/build/Release/src/starkware/starknet/apps/starkgate/starkgate_flow_test")
set_tests_properties([=[starkgate_flow_test]=] PROPERTIES  _BACKTRACE_TRIPLES "/app/src/cmake_utils/python_rules.cmake;247;add_test;/app/src/cmake_utils/python_rules.cmake;298;python_test;/app/src/starkware/starknet/apps/starkgate/CMakeLists.txt;27;full_python_test;/app/src/starkware/starknet/apps/starkgate/CMakeLists.txt;0;")
subdirs("cairo")
subdirs("eth")
