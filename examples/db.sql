# Get monthly total by user and limits
select config.payLimit, sum(counts.quantity) as monthlyTotal
from config,counts
where config.username='user'
    AND counts.username='user'
    AND txDate > '2022-08-00' 
    AND txDate < '2022-09-00';