# Contiguous Memory Allocation

Instructions:
https://cs.winona.edu/Nguyen/fa23/CS405/projects/CS%20405%20FA23%20Project%202.pdf

CS 405 - Fall 2023
Project 2: Contiguous Memory Allocation :tada:
---

## 1. Objectives <br>
This teamwork project is designed to implement `a program in any language, C/C++ or Java` that demonstrates the continuous memory allocation schemes: <br>

    - Apply next-fit, best-fit, and worst-fit strategies for allocating memory contiguously.
    - Understand the distinction between internal and external fragmentation.


## 2. Overview <br>
Your team needs to design and implement a program (`in any language, C/C++ or Java`)
that simulates the continuous memory allocation to processes using best-fit, worst-fit,
and next-fit strategies. <br>
```
  - Note: First-fit is not included in this project because we will implement it in lab#6. Next
Fit algorithm is a modification of the First Fit algorithm. Next, fit scans the linked list from
the node where it previously allocated a hole to the requested process (instead of
scanning the list from the beginning like First-fit). The idea behind the next fit is the fact
that the list has been scanned once therefore, the probability of finding the hole is larger
in the remaining part of the list. Definitely, if Next-fit cannot find a suitable hole, it needs
to go back to the beginning of the list to scan.
This project will involve managing a contiguous region of memory of size MAX where
addresses may range from 0 ... MAX−1. Your program must support the following
operations of a memory management simulation:
```
    - Allocate a memory partition to a process.
    - Free a memory partition allocated to a process, and this process is terminated
    - Report the current state of the memory: allocated memory partitions and free holes

## 3. Requirement Specs <br>
## 3.1 General Requirements

    - This is a teamwork project in which you need to form a team of a maximum of 3 persons.
    - You can continue with your team in Project1 or start a new team.
    - You can implement the simulation program in any language of your choice (C/C++,Java,Python)

## 3.2 Input Format <br>
When your program starts, you will read a configuration file in the following format. This file will contain multiple rows, each in the format `<key>=<value>`.
The following table lists different keys used in this program: 
![image](https://github.com/Pickles91/ContiguousMemoryAllocation/assets/46804029/61304c04-6231-406b-b239-b853a814796e)

These configuration parameters will be used throughout your program to manage the number of processes, memory boundary, and handle the request/release memory partitions from processes. 

## 3.3 Main program <br>
You can develop your memory allocation simulator program in Console mode or GUI mode.
The main application class should perform the following tasks: 
  - Load the configuration file -check the previous section (3.2) for theconfiguration format.
  - Generate `NUM_PROC` number of processes with random process size and lifetime.
  - Service the memory request and release for each processor memory compaction (extracredit)
  - Display the simulation statistics result.

## 3.3.1 Generate random processes <br>
After loading the configuration file, your program will generate the list of processes with `NUM_PROC` size. Each process will need a random size between `0-PROC_SIZE_MAX` and a random lifetime between `0-MAX_PROC_TIME` in ms. The lifetime of a process is between the time the process is allocated to a memory partition (`match with its request size`) and the time the process terminates and releases its memory. <br>

## 3.3.2 Allocating Deallocating Memory <br>
Your program will allocate memory using one of the three approaches highlighted in Section 9.2.2: `best-fit`, `worst-fit`, and `next-fit`. Your program will try to allocate as many processes as possible in the sequence of the generated processes in step 3.3.1. Any processes whose request cannot be fulfilled have to wait. 

This will require that your program keep track of the different holes representing available memory. When a request for a memory of a process arrives, it will allocate the memory from one of the available holes based on the allocation strategy. If there is insufficient memory to allocate to a request, your program will output an error message and put that request on hold until current running processes terminate and free their memory. 

Your program will also need to keep track of which region of memory has been allocated to which process. This is necessary to support the memory visualization function(3.3.3) and is also needed when memory is released when a process terminates.If a partition being released is adjacent to an existing hole, combine the two holes into a single hole.

## 3.3.3 The simulation visualization <br>  
Your program should keep track of the below simulation status events and display them:

```
- Your program will display the regions of memory that are allocated and the regions that are unused after each execution step of all processes every second.
```

Below is a sample text representation on the Console mode: 

```
|P1[15s](30KB)|P3[12s](10KB)|Free(40KB)|…|
```

The above example displays three memory partitions: 
    a) The first memory partition has a size of 30 KB is allocated to P1. This             process will continue to run for the next 15 s before terminating
    b) The second memory partition has a size of 10 KB and is allocated to P3. This
       process will continue to run for the next 12 s before terminating
    c)The third memory partition is a free (hole) and has a size of 40 KB 
    
Here is another example of a program in the GUI mode: 
![image](https://github.com/Pickles91/ContiguousMemoryAllocation/assets/46804029/94bebe98-20f7-4114-84d0-12e3bf0f9bd0)

The above figure displays the memory allocation status for 9 processes from P1 to P9 for three allocation schemes (drawn with light yellow background color). There is 1 free memory partition (a hole that is drawn with a light blue background color) with a size of 175 KB, which is not enough to allocate to the next process, P10 (request 179 KB). The number in the parentheses next to the process ID is the remaining running time of the process in seconds (if it reduces to 0 => the process terminates and releases its memory).

    - Display the memory allocation statistics for each execution step (every second): 
    + The number of holes (free memory partitions)
    + The average size of the current list of holes + The total size of the current list of holes 
    + The percentage of total free memory partition over the total memory 

Check the above figure for the sample statistics output. 


## 3.4 Testing and Reporting the Results <br>

    -After completing the implementation of your program, you may need to test your program with different settings in the configuration file.
    -Finally, write a report to discuss your results. 

    
## 3.5 Extra Credits <br> 
Your team can earn extra credits for this project if you can implement the following extra works:

    -(10pts) Implement the memory compaction function in your program. Your program will compact the set of holes into one larger hole. 
    For example, if you have four separate holes of size 550 KB, 375 KB, 1, 900 KB,
    and 4,500 KB, your program will combine these four holes in to one large hole of size 7, 325 KB. 
    There are several strategies for implementing compaction, one of which is suggested in Section 9.2.3. 
    Be sure to update the beginning address of any processes affected by compaction.You can implement this function by entering a specific key on the Console or clicking a button if you use GUI to developy our application. 
    
## 4. Project Deliverables <br>
You need to submit the following items: 

    1. Compress the source code of your project, test data (scenario files), test outputs, and comparison results. 
    2. A DOCX/PDF report (less than ten pages) to summarize the implementation details of your team, discussion, individual contributions of each member, and references. Submit two above items to D2L. 
    
## 5.GradingRubrics <br>
## 5.1 Program design and implementation (80pts) <br>
 
    - 20 pts: Correct implementation of the main program that takes input arguments, start/pause/resume, or stop the simulation
    - 30 pts: Correct implementation of the three memory allocation algorithms:    first-fit, best-fit, and worst-fit
    - 20 pts: Correct display different events in the simulation: memory
    partitions allocated to a process, free memory from a terminated process, and the memory allocations statistics. 
    - 10 pts: Good data structures and good coding style and comments 

## 5.2 Report (20pts) <br> 
    - 10 pts: Present your simulation results with different configuration settings (at least four) 
    - 5 pts: Discuss your finding of the simulation and lessons
    - 5 pts: Report how your team works together, individual contributions, and references  
## 5.3 Extra Credits (maximum 10 pts if applicable)
    - 10 pts: Correctly implement the memory compaction function.

