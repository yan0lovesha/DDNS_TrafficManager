# DDNS by Azure Traffic Manager

This Rust program updates an Azure Traffic Manager Profile endpoint with the public IP address of the machine running the program. 

My home network doesn't have a static IP. So the public IP for my home network changes as it want. I want to be able access my home network without checking the public IP on my router everytime.  I know that there are DDNS service. But I don't want to pay for owning a domain name. The most DDNS service are also not free or conditional free. So, I come up this solution by using Azure Traffic Manager Profile. The price of it can be ignored for most of the individual users. Detailed pricing info can be found [here]([url](https://azure.microsoft.com/en-us/pricing/details/traffic-manager/)). This program simply get the public IP of your home network, update the specified Azure Traffic Manager to point to this public IP. So, you can use the domain name of the Traffic Manager to access your home network.

I use it on Raspberry pi. If you want to use it on a different platform, you could build it for your own target.

## Usage

### Prerequisites

- Rust installed on your system
- An Azure subscription (Free). You ened to note the **TenantId** and **SubscriptionId** for later use.
- A Traffic Manager Profile Resource (Free). You need to note the **ResourceGroup name** and the **Traffic Manager Profile instance name** for later use.
- An external endpoint created in the Traffic Manager Profile (Free). You can set a fake IP like 1.1.1.1 here for now. You need to note the **Endpoint name** and **Endpoint Location" for later user.
- An Entra Application with a secret. You need to note the **Application client id** and the **secret** for later use.
- In Traffic Manager's IAM settings, grant your Entra Application a contributor role.

### Build and deploy the project

The target for Raspberry Pi 64bit is: aarch64-unknown-linux-gnu
```
cargo build --release --target [Your target]
```

Copy the compiled program to your target machine which is always on in your home network.

if you want to build and deploy the compiled program to the Raspberry pi, a shell script is provide to do so. Run:
```
./build [user name] [IP or Host Name of Pi] [Port]
```
E.g.
```
./build piadmin 192.168.0.33 22
```

### Create a service to run the Program
sudo nano /etc/systemd/system/DDNS.service 

Place the content below in it. 
```
[Unit] 
Description=DDNS Service 
After=network.target 
Wants=DDNS.timer 
 
[Service] 
Type=oneshot 
ExecStart=/[Path to the directory of your program]/DDNS_TrafficManager 
WorkingDirectory=/[Path to the directory of your program] -tenant_id "your_tenant_id" --subscription_id "your_subscription_id" --resource_group "your_resource_group" --traffic_manager_name "your_traffic_manager_name" --endpoint_name "your_endpoint_name" --endpoint_location "your_endpoint_location" --client_id "your_client_id" --client_secret "your_client_secret"
 
[Install] 
WantedBy=multi-user.target 
```

### Create a timer service that triggers the previouse service
sudo nano /etc/systemd/system/DDNS.timer 

Place the content below in it. You can change the OnCalendar value to run it in on different time interval. Currently it runs every 5 minutes.
```
[Unit] 
Description=DDNS Timer 
Requires=DDNS.service 
 
[Timer] 
Unit=DDNS.service 
OnCalendar=*:0/5 
 
[Install] 
WantedBy=timers.target 
```
### Start the timer service
```
sudo systemctl daemon-reload 
sudo systemctl enable DDNS.timer 
sudo systemctl start DDNS.timer
```

If you want to stop the timer, run below cmd in shell.
```
sudo systemctl stop DDNS.timer 
```

### Verify it is working
Run below command in bash to verify if the timer service runs successfully.
```
sudo systemctl status DDNS.timer
```

Go to Azure portal and your TrafficManager. Check the IP address of your Endpoint. It should no longer the fake IP you gave it.

Try to ping your Traffic Manager domain name to see if it is resolved to your home network IP address.
```
ping [TrafficManagerName].trafficmanager.net
```
