# CMAKE generated file: DO NOT EDIT!
# Generated by "Unix Makefiles" Generator, CMake Version 3.22

# Delete rule output on recipe failure.
.DELETE_ON_ERROR:

#=============================================================================
# Special targets provided by cmake.

# Disable implicit rules so canonical targets will work.
.SUFFIXES:

# Disable VCS-based implicit rules.
% : %,v

# Disable VCS-based implicit rules.
% : RCS/%

# Disable VCS-based implicit rules.
% : RCS/%,v

# Disable VCS-based implicit rules.
% : SCCS/s.%

# Disable VCS-based implicit rules.
% : s.%

.SUFFIXES: .hpux_make_needs_suffix_list

# Command-line flag to silence nested $(MAKE).
$(VERBOSE)MAKESILENT = -s

#Suppress display of executed commands.
$(VERBOSE).SILENT:

# A target that is always out of date.
cmake_force:
.PHONY : cmake_force

#=============================================================================
# Set environment variables for the build.

# The shell in which to execute make rules.
SHELL = /bin/sh

# The CMake executable.
CMAKE_COMMAND = /usr/local/lib/python3.7/site-packages/cmake/data/bin/cmake

# The command to remove a file.
RM = /usr/local/lib/python3.7/site-packages/cmake/data/bin/cmake -E rm -f

# Escaping for special characters.
EQUALS = =

# The top-level source directory on which CMake was run.
CMAKE_SOURCE_DIR = /app

# The top-level build directory on which CMake was run.
CMAKE_BINARY_DIR = /app/build/Release

# Utility rule file for proxy_contract_sol.

# Include any custom commands dependencies for this target.
include src/starkware/solidity/upgrade/CMakeFiles/proxy_contract_sol.dir/compiler_depend.make

# Include the progress variables for this target.
include src/starkware/solidity/upgrade/CMakeFiles/proxy_contract_sol.dir/progress.make

src/starkware/solidity/upgrade/CMakeFiles/proxy_contract_sol: python_libs/proxy_contract_sol.info

python_libs/proxy_contract_sol.info: ../../src/cmake_utils/gen_py_lib.py
python_libs/proxy_contract_sol.info: python_libs/common_library_sol.info
python_libs/proxy_contract_sol.info: python_libs/governance_contract_sol.info
python_libs/proxy_contract_sol.info: python_libs/solidity_contracts_interfaces_sol.info
python_libs/proxy_contract_sol.info: src/starkware/solidity/upgrade/proxy_contract_sol_copy_files.stamp
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --blue --bold --progress-dir=/app/build/Release/CMakeFiles --progress-num=$(CMAKE_PROGRESS_1) "Generating ../../../../python_libs/proxy_contract_sol.info"
	cd /app/build/Release/src/starkware/solidity/upgrade && ../../../../../../src/cmake_utils/gen_py_lib.py --name proxy_contract_sol --lib_dir /app/build/Release/src/starkware/solidity/upgrade/proxy_contract_sol --files starkware/solidity/upgrade/Proxy.sol starkware/solidity/upgrade/ProxyGovernance.sol starkware/solidity/upgrade/ProxyStorage.sol starkware/solidity/upgrade/StorageSlots.sol --lib_deps common_library_sol governance_contract_sol solidity_contracts_interfaces_sol --py_exe_deps --cmake_dir src/starkware/solidity/upgrade --prefix starkware/solidity/upgrade/ --output /app/build/Release/python_libs/proxy_contract_sol.info

proxy_contract_sol: python_libs/proxy_contract_sol.info
proxy_contract_sol: src/starkware/solidity/upgrade/CMakeFiles/proxy_contract_sol
proxy_contract_sol: src/starkware/solidity/upgrade/CMakeFiles/proxy_contract_sol.dir/build.make
.PHONY : proxy_contract_sol

# Rule to build all files generated by this target.
src/starkware/solidity/upgrade/CMakeFiles/proxy_contract_sol.dir/build: proxy_contract_sol
.PHONY : src/starkware/solidity/upgrade/CMakeFiles/proxy_contract_sol.dir/build

src/starkware/solidity/upgrade/CMakeFiles/proxy_contract_sol.dir/clean:
	cd /app/build/Release/src/starkware/solidity/upgrade && $(CMAKE_COMMAND) -P CMakeFiles/proxy_contract_sol.dir/cmake_clean.cmake
.PHONY : src/starkware/solidity/upgrade/CMakeFiles/proxy_contract_sol.dir/clean

src/starkware/solidity/upgrade/CMakeFiles/proxy_contract_sol.dir/depend:
	cd /app/build/Release && $(CMAKE_COMMAND) -E cmake_depends "Unix Makefiles" /app /app/src/starkware/solidity/upgrade /app/build/Release /app/build/Release/src/starkware/solidity/upgrade /app/build/Release/src/starkware/solidity/upgrade/CMakeFiles/proxy_contract_sol.dir/DependInfo.cmake --color=$(COLOR)
.PHONY : src/starkware/solidity/upgrade/CMakeFiles/proxy_contract_sol.dir/depend

