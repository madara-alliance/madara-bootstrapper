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

# Utility rule file for solidity_contracts_interfaces_sol_copy_files.

# Include any custom commands dependencies for this target.
include src/starkware/solidity/interfaces/CMakeFiles/solidity_contracts_interfaces_sol_copy_files.dir/compiler_depend.make

# Include the progress variables for this target.
include src/starkware/solidity/interfaces/CMakeFiles/solidity_contracts_interfaces_sol_copy_files.dir/progress.make

src/starkware/solidity/interfaces/CMakeFiles/solidity_contracts_interfaces_sol_copy_files: src/starkware/solidity/interfaces/solidity_contracts_interfaces_sol_copy_files.stamp

src/starkware/solidity/interfaces/solidity_contracts_interfaces_sol_copy_files.stamp: src/starkware/solidity/interfaces/solidity_contracts_interfaces_sol/starkware/solidity/interfaces/BlockDirectCall.sol
src/starkware/solidity/interfaces/solidity_contracts_interfaces_sol_copy_files.stamp: src/starkware/solidity/interfaces/solidity_contracts_interfaces_sol/starkware/solidity/interfaces/IFactRegistry.sol
src/starkware/solidity/interfaces/solidity_contracts_interfaces_sol_copy_files.stamp: src/starkware/solidity/interfaces/solidity_contracts_interfaces_sol/starkware/solidity/interfaces/Identity.sol
src/starkware/solidity/interfaces/solidity_contracts_interfaces_sol_copy_files.stamp: src/starkware/solidity/interfaces/solidity_contracts_interfaces_sol/starkware/solidity/interfaces/IQueryableFactRegistry.sol
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --blue --bold --progress-dir=/app/build/Release/CMakeFiles --progress-num=$(CMAKE_PROGRESS_1) "Generating solidity_contracts_interfaces_sol_copy_files.stamp"
	cd /app/build/Release/src/starkware/solidity/interfaces && /usr/local/lib/python3.7/site-packages/cmake/data/bin/cmake -E touch /app/build/Release/src/starkware/solidity/interfaces/solidity_contracts_interfaces_sol_copy_files.stamp

src/starkware/solidity/interfaces/solidity_contracts_interfaces_sol/starkware/solidity/interfaces/BlockDirectCall.sol: ../../src/starkware/solidity/interfaces/BlockDirectCall.sol
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --blue --bold --progress-dir=/app/build/Release/CMakeFiles --progress-num=$(CMAKE_PROGRESS_2) "Copying file BlockDirectCall.sol"
	cd /app/build/Release/src/starkware/solidity/interfaces && /usr/local/lib/python3.7/site-packages/cmake/data/bin/cmake -E copy /app/src/starkware/solidity/interfaces/BlockDirectCall.sol /app/build/Release/src/starkware/solidity/interfaces/solidity_contracts_interfaces_sol/starkware/solidity/interfaces/BlockDirectCall.sol

src/starkware/solidity/interfaces/solidity_contracts_interfaces_sol/starkware/solidity/interfaces/IFactRegistry.sol: ../../src/starkware/solidity/interfaces/IFactRegistry.sol
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --blue --bold --progress-dir=/app/build/Release/CMakeFiles --progress-num=$(CMAKE_PROGRESS_3) "Copying file IFactRegistry.sol"
	cd /app/build/Release/src/starkware/solidity/interfaces && /usr/local/lib/python3.7/site-packages/cmake/data/bin/cmake -E copy /app/src/starkware/solidity/interfaces/IFactRegistry.sol /app/build/Release/src/starkware/solidity/interfaces/solidity_contracts_interfaces_sol/starkware/solidity/interfaces/IFactRegistry.sol

src/starkware/solidity/interfaces/solidity_contracts_interfaces_sol/starkware/solidity/interfaces/IQueryableFactRegistry.sol: ../../src/starkware/solidity/interfaces/IQueryableFactRegistry.sol
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --blue --bold --progress-dir=/app/build/Release/CMakeFiles --progress-num=$(CMAKE_PROGRESS_4) "Copying file IQueryableFactRegistry.sol"
	cd /app/build/Release/src/starkware/solidity/interfaces && /usr/local/lib/python3.7/site-packages/cmake/data/bin/cmake -E copy /app/src/starkware/solidity/interfaces/IQueryableFactRegistry.sol /app/build/Release/src/starkware/solidity/interfaces/solidity_contracts_interfaces_sol/starkware/solidity/interfaces/IQueryableFactRegistry.sol

src/starkware/solidity/interfaces/solidity_contracts_interfaces_sol/starkware/solidity/interfaces/Identity.sol: ../../src/starkware/solidity/interfaces/Identity.sol
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --blue --bold --progress-dir=/app/build/Release/CMakeFiles --progress-num=$(CMAKE_PROGRESS_5) "Copying file Identity.sol"
	cd /app/build/Release/src/starkware/solidity/interfaces && /usr/local/lib/python3.7/site-packages/cmake/data/bin/cmake -E copy /app/src/starkware/solidity/interfaces/Identity.sol /app/build/Release/src/starkware/solidity/interfaces/solidity_contracts_interfaces_sol/starkware/solidity/interfaces/Identity.sol

solidity_contracts_interfaces_sol_copy_files: src/starkware/solidity/interfaces/CMakeFiles/solidity_contracts_interfaces_sol_copy_files
solidity_contracts_interfaces_sol_copy_files: src/starkware/solidity/interfaces/solidity_contracts_interfaces_sol/starkware/solidity/interfaces/BlockDirectCall.sol
solidity_contracts_interfaces_sol_copy_files: src/starkware/solidity/interfaces/solidity_contracts_interfaces_sol/starkware/solidity/interfaces/IFactRegistry.sol
solidity_contracts_interfaces_sol_copy_files: src/starkware/solidity/interfaces/solidity_contracts_interfaces_sol/starkware/solidity/interfaces/IQueryableFactRegistry.sol
solidity_contracts_interfaces_sol_copy_files: src/starkware/solidity/interfaces/solidity_contracts_interfaces_sol/starkware/solidity/interfaces/Identity.sol
solidity_contracts_interfaces_sol_copy_files: src/starkware/solidity/interfaces/solidity_contracts_interfaces_sol_copy_files.stamp
solidity_contracts_interfaces_sol_copy_files: src/starkware/solidity/interfaces/CMakeFiles/solidity_contracts_interfaces_sol_copy_files.dir/build.make
.PHONY : solidity_contracts_interfaces_sol_copy_files

# Rule to build all files generated by this target.
src/starkware/solidity/interfaces/CMakeFiles/solidity_contracts_interfaces_sol_copy_files.dir/build: solidity_contracts_interfaces_sol_copy_files
.PHONY : src/starkware/solidity/interfaces/CMakeFiles/solidity_contracts_interfaces_sol_copy_files.dir/build

src/starkware/solidity/interfaces/CMakeFiles/solidity_contracts_interfaces_sol_copy_files.dir/clean:
	cd /app/build/Release/src/starkware/solidity/interfaces && $(CMAKE_COMMAND) -P CMakeFiles/solidity_contracts_interfaces_sol_copy_files.dir/cmake_clean.cmake
.PHONY : src/starkware/solidity/interfaces/CMakeFiles/solidity_contracts_interfaces_sol_copy_files.dir/clean

src/starkware/solidity/interfaces/CMakeFiles/solidity_contracts_interfaces_sol_copy_files.dir/depend:
	cd /app/build/Release && $(CMAKE_COMMAND) -E cmake_depends "Unix Makefiles" /app /app/src/starkware/solidity/interfaces /app/build/Release /app/build/Release/src/starkware/solidity/interfaces /app/build/Release/src/starkware/solidity/interfaces/CMakeFiles/solidity_contracts_interfaces_sol_copy_files.dir/DependInfo.cmake --color=$(COLOR)
.PHONY : src/starkware/solidity/interfaces/CMakeFiles/solidity_contracts_interfaces_sol_copy_files.dir/depend

