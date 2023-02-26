#############
Nano Services
#############

Nano Services is an upgraded version of micro-services.

Instead of having each *service* as a separate app, let's make each *service* a thread.
This allows for efficient move operations between *services* (no large copies).

Instead of having to create a message passing protocol between threads, let's use the public methods as the thread boundary.
This allows for inter-thread code that just feels like calling methods, because it is.
No worrying about drafting a communication protocol between threads.
And this is still efficient because of the move operations between threads.

So if each *service* is a thread and threads communicate via methods, that means that each 

Each struct instance gets its own thread. It communicate via the function calls of the struct.

A macro generates a thread for each struct instance and uses its public methods as the interface between the different threads.



It can also be seen as more convenient version of event driven design.


.. contents:: **Table of Contents**
    :depth: 2
.. section-numbering::

******************
Overview
******************

*Text Here*

====================
Purpose
====================

When writing micro-services, each service has one responsibility. Each service then communicates with other services to accomplish larger goals.
One example may be to run a database as one service and the main program controller as another service.
This allows for the separation of responsibility, fault isolation, easier scalability, and more.

There is still room for improvement though. 
Copying data to send across process boundaries can be very inefficient (especially when using network traffic).
There may also be more difficulty debugging micro-services due to difficulties with reproducibility.


Also, when writing code with event-driven design, many different events must be created that may only be loosely coupled to specific program functions. 
By using methods as the interface, between 

====================
How It Works
====================

Nano Services solves this problem by using using methods as the basis for events.



******************
Build Instructions
******************

====================
Clone the repository
====================

.. code:: bash

    git clone git@github.com:bruskajp/nano_services
    git submodule update --init --recursive

============
Build  
============



**********************
How To Use The Library
**********************

=====
Setup
=====


======
Use it
======


=====================
Design Considerations
=====================


*************
FAQ
*************

#. Who should you ask almost any question about this code to?

   * James Bruska

*************
License
*************

Nano Services is (c) 2022 by James Bruska, and licensed as open source under the GPLv3, with the full details in LICENSE.txt. 

****************
More Information
****************

Please see the docs folder


