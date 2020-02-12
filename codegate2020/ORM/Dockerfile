FROM ubuntu:18.04

# Setup environ
ENV user orm
ENV prob_port 31337

# Install packages
RUN sed -i "s/http:\/\/archive.ubuntu.com/http:\/\/kr.archive.ubuntu.com/g" /etc/apt/sources.list
RUN apt-get update
RUN apt-get install -y xinetd

# Change permission
RUN chmod 1733 /tmp /var/tmp /dev/shm

# Additional configuration
RUN adduser --disabled-login --gecos '' $user
ADD ./bins/orm /home/$user/orm
ADD ./bins/chal.ormb /home/$user/chal.ormb
ADD ./bins/orm-flag /home/$user/flag
ADD ./bins/run.sh /home/$user/run.sh
ADD ./xinetd.conf /etc/xinetd.d/chal

RUN chown -R root:root /home/$user/
RUN chown root:$user /home/$user/orm
RUN chown root:$user /home/$user/chal.ormb
RUN chown root:$user /home/$user/flag
RUN chown root:$user /home/$user/run.sh

RUN chmod 2755 /home/$user/orm
RUN chmod 440 /home/$user/chal.ormb
RUN chmod 440 /home/$user/flag
RUN chmod 2755 /home/$user/run.sh

# RUN xinetd
EXPOSE $prob_port
RUN service xinetd restart
CMD ["/usr/sbin/xinetd", "-dontfork"]
