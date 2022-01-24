FROM python

RUN pip3 install pip -U
RUN pip3 config set global.index-url https://pypi.tuna.tsinghua.edu.cn/simple
RUN pip3 install numpy sympy scipy matplotlib

COPY sources.list /etc/apt/
RUN apt update 
RUN apt install gcc g++ -y
RUN apt install dvipng -y
RUN apt install texlive-full -y
RUN pip3 install pillow
RUN apt install ghc -y
RUN apt install rustc -y
COPY check.py /
RUN python /check.py