---
title: "📃 Academic Bibliography"
description: "The academic bibliography."
date: 2026-06-22
---

{{< katex >}}

Software birthmarking has been researched extensively. Below is a collection of academic papers detailing the concepts, algorithms, and implementations used by this toolkit.

## 📜 Publications by Haruaki Tamada (Creator)

1. **Cross-Platform Software Birthmarking for Real-World Binaries via Intermediate Representation**, *Haruaki Tamada*, In Proc. 34th IEEE/ACIS International Conference on Software Engineering, Artificial Intelligence, Networking and Parallel/Distributed Computing ([SNPD2026](https://acisinternational.org/conferences/snpd-2026-i/)), August 2026 (Okayama, Japan, submitted).　[ [arXiv](https://arxiv.org/abs/2606.21988) ]
  * **Contribution:** Proposed a cross-platform birthmarking approach using a Ghidra P-code intermediate representation, demonstrating exceptional architecture consistency (\\(r=0.9846\\)) and identifying the Simpson index's resilience against library dilution.

2. **Comparison of Similarity Functions for n-gram Software Birthmarks**, *Nikolay Fedorov, Hiroki Inayoshi, Haruaki Tamada, and Akito Monden*, In Proc. 6th World Symposium on Software Engineering ([WSSE 2024](https://wsse.org/2024.html)), pp. 169--176, September 2024 (Kyoto, Japan).　[ [Link](https://dl.acm.org/doi/10.1145/3698062.3698087) ]
  * **Contribution:** Conducted exhaustive comparison of various similarity algorithms on \\(n\\)-gram opcode software birthmarks.

3. **Improvement of the Dynamic Software Birthmark Process by Reducing the Time of the Extraction**, *Takanori Yokoi, and Haruaki Tamada*, International Journal of Networked and Distributed Computing, Vol. 6, Issue 4, pp. 224--231, September 2018. [ [Link](https://www.atlantis-press.com/journals/ijndc/125905554) ]
  * **Contribution:** Proposed using unit tests as the execution driver to reduce dynamic birthmark extraction time while maintaining high credibility and resilience.

4. **mituba: Scaling up Software Theft Detection with the Search Engine**, *Jun Nakamura and Haruaki Tamada*, In Proc. International Conference on Software Engineering and Information Management ([ICSIM 2018](https://icsim.org/icsim2018.html)), pp. 6--10, January 2018 (Casablanca, Morocco). [ [Link](https://dl.acm.org/doi/10.1145/3178461.3178475) ]
  * **Contribution:** Improved and scaled up the search-engine-based comparison method proposed in ACIT 2016.

5. **Fast Comparison of Software Birthmarks for Detecting the Theft with the Search Engine**, *Jun Nakamura and Haruaki Tamada*, In Proc. of the 4th International Conference on Applied Computing & Information Technology ([ACIT 2016](https://www.computer.org/csdl/proceedings/acit-csi/2016/12OmNBscCYB)), pp. 152--157, December 2016 (UNLV, Las Vegas, NV, USA). [ [Link](https://ieeexplore.ieee.org/document/7916974/) ]
  * **Contribution:** Demonstrated using a search engine index to rapidly retrieve and match similar birthmarks.

6. **A Fuzzy Hashing Technique for Large Scale Software Birthmarks**, *Takehiro Tsuzaki, Teruaki Yamamoto, Haruaki Tamada, and Akito Monden*, In Proc. 15th IEEE/ACIS International Conference on Computer and Information Science ([ICIS 2016](https://www.computer.org/csdl/proceedings/2016/icis/12OmNxuXcvH)), pp. 867--872, July 2016 (Okayama, Japan).  [ [Link](https://ieeexplore.ieee.org/document/7550868) ]
  * **Contribution:** Introduced fuzzy hashing to significantly accelerate software birthmark comparisons.

7. **A Dynamic Birthmark from Analyzing Operand Stack Runtime Behavior to Detect Copied Software**, *Kazumasa Fukuda, and Haruaki Tamada*, In Proc. 13th ACIS International Conference on Software Engineering, Artificial Intelligence, Networking and Parallel/Distributed Computing ([SNPD 2013](https://dl.acm.org/doi/proceedings/10.5555/2553191)), pp. 505--510, July 2013 (Honolulu, Hawaii, U.S.A.). [ [Link](https://ieeexplore.ieee.org/document/6598511/) ]
  * **Contribution:** Proposed a dynamic software birthmarking method for Java based on the runtime behavior and state transitions of the JVM operand stack.

8. **Using Software Birthmarks to Identify Similar Classes and Major Functionalities**, *Takesi Kakimoto, Akito Monden, Yasutaka Kamei, Haruaki Tamada, Masateru Tsunoda, and Ken-ichi Matsumoto*, In Proc. the 3rd International Workshop on Mining Software Repositories ([MSR 2006](http://msr.uwaterloo.ca/msr2006/)), pp. 171--172, May 2006 (Shanghai, China). [ [Link](https://dl.acm.org/doi/10.1145/1137983.1138026) ]
  * **Contribution:** Explored a novel application of software birthmarks to automatically group similar classes and identify major functional modules in large codebases.

9. **Java Birthmarks --Detecting the Software Theft--**, *Haruaki Tamada, Masahide Nakamura, Akito Monden, and Ken'ichi Matsumoto*, IEICE Transactions on Information and Systems, Vol. E88-D, No. 9, pp. 2148--2158, September 2005. [ [Link](https://dl.acm.org/doi/10.1093/ietisy/e88-d.9.2148) ]
  * **Contribution:** Formally established the static software birthmarking framework.

10. **Dynamic Software Birthmarks to Detect the Theft of Windows Applications**, *Haruaki Tamada, Keiji Okamoto, Masahide Nakamura, Akito Monden, and Kenichi Matsumoto*, In Proc. International Symposium on Future Software Technology 2004 (ISFST 2004), October 2004 (Xi'an, China).  [ [Link](https://www.semanticscholar.org/paper/Dynamic-Software-Birthmarks-to-Detect-the-Theft-of-Tamada-Okamoto/44085ac534b0120ad516f9f61ad0901bd360ef18) ]
  * **Contribution:** Introduced dynamic software birthmarks, and proposed birthmark types EXESEQ (Execution Sequence) and EXEFREQ (Execution Frequency).

11. **Design and Evaluation of Birthmarks for Detecting Theft of Java Programs**, *Haruaki Tamada, Masahide Nakamura, Akito Monden, and Kenichi Matsumoto*, In Proc. IASTED International Conference on Software Engineering (IASTED SE 2004), pp. 569--575, February 2004 (Innsbruck, Austria). [ [Link](https://www.academia.edu/4149089/Design_and_evaluation_of_birthmarks_for_detecting_theft_of_java_programs) ]
  * **Contribution:** Proposed the foundational concept of software birthmarks, introducing the static birthmark types CVFV (Constant Value Frequency Vector), UC (Used Classes), SMC (Sequence of Method Calls), and IS (Inheritance Structure).

---

## 📄 Other Fundamental Papers on Software Birthmarks

### \\(k\\)-gram-based birthmarks

* **\\(k\\)-gram-based software birthmarks**, *Ginger Myles and Christian Collberg*, In Proc. of the 2005 ACM Symposium on Applied Computing, pp. 314--318, March 2005. [ [Link](https://dl.acm.org/doi/10.1145/1066677.1066753) ]
  * **Summary:** Introduced the extraction of instruction sequences chunked into sliding windows (\\(k\\)-grams) for highly robust program similarities.

### Whole Program Path (Dynamic Birthmarks)

* **Detecting Software Theft via Whole Program Path Birthmarks**, *Ginger Myles and Christian Collberg*, In Proc. of the International Conference on Information Security 2004, pp. 404--415, 2004.  
  * **Summary:** Proposed dynamic birthmarking based on whole program execution paths.

---

## 📚 Systematic Surveys and Books

* **Surreptitious Software: Obfuscation, Watermarking, and Tamperproofing for Software Protection**, *Christian Collberg and Jasvir Nagra*, Addison-Wesley Professional, ISBN: 978-0-321-54925-9, August 2009.  
  * **Summary:** The leading comprehensive book covering software security, obfuscation, watermarking, and birthmarking techniques.
  
* **Software Birthmark Design and Estimation: A Systematic Literature Review**, *Shah Nazir, Sara Shahzad and Neelam Mukhtar*, Arabian Journal for Science and Engineering, Vol. 44, pp. 3905--3927, January 2019.  
  * **Summary:** A comprehensive review mapping out the state-of-the-art developments and methodologies in software birthmarking.
